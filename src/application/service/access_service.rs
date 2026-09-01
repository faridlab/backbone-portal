//! `AccessService` — the principal access verbs: throttled login (the
//! credential port), the policy-gated signup, and the revocation sweeps
//! (hand-written; user-owned; see `metaphor.codegen.yaml`).
//!
//! ## The de-oracle rule (PI-23)
//!
//! Login failures share ONE answer ([`PortalError::InvalidCredentials`]):
//! unknown email, wrong credential (the port's `Ok(false)` — the port's
//! contract makes the two indistinguishable), and a revoked/archived
//! principal. Which half failed is recorded NOWHERE — not in the
//! response, not in the audit row (the `login_refused` audit row records
//! the email + envelope facts only).
//!
//! ## The throttle (Tier B, ADR-0018)
//!
//! Every login/signup attempt rides the per-identity AND per-IP
//! escalating lockout book (the survey session-code curve, pure + probe
//! asserted): 5 failures -> 30 s doubling to a 15 min cap, 1 s minimum
//! spacing, success resets. The lockout refusal (429) is typed and
//! deliberately NOT part of the shared 401 body — a throttled caller
//! already knows their own attempt rate.
//!
//! ## Signup (PI-09/PI-18)
//!
//! `signup` re-reads the kill switch on EVERY call: closed (the default
//! posture) refuses with the typed [`PortalError::SignupClosed`] +
//! an audited `signup_refused_policy_closed` row; open creates the
//! principal through the same credential check as login (the port) and
//! mints the first bearer. The flip bites immediately — no cached read.

use std::sync::Arc;

use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::application::service::credential_port::{CredentialCheck, CredentialVerifierSlot};
use crate::application::service::invite_service::normalize_email;
use crate::application::service::portal_error::PortalError;
use crate::application::service::policy_service::PolicyService;
use crate::application::service::token_service::{
    AttemptBook, TokenService, DEFAULT_BEARER_TTL_HOURS,
};

/// The access verb engine.
pub struct AccessService {
    pool: PgPool,
    tokens: Arc<TokenService>,
    policy: PolicyService,
    credentials: CredentialVerifierSlot,
    attempts: Arc<AttemptBook>,
}

impl AccessService {
    pub fn new(
        pool: PgPool,
        tokens: Arc<TokenService>,
        policy: PolicyService,
        credentials: CredentialVerifierSlot,
    ) -> Self {
        let attempts = tokens.attempt_book();
        Self { pool, tokens, policy, credentials, attempts }
    }

    pub fn credential_slot(&self) -> CredentialVerifierSlot {
        self.credentials.clone()
    }

    fn identity_key(email: &str) -> String {
        format!("portal|id:{}", normalize_email(email))
    }

    fn ip_key(ip: &str) -> String {
        format!("portal|ip:{ip}")
    }

    async fn audit(
        &self,
        event: &str,
        user: Option<Uuid>,
        actor: &str,
        detail: serde_json::Value,
    ) -> Result<(), PortalError> {
        sqlx::query(
            r#"INSERT INTO portal.portal_audit_log
                 (id, event, portal_user_id, actor, detail)
               VALUES ($1, $2::portal_audit_event, $3, $4, $5)"#,
        )
        .bind(Uuid::new_v4())
        .bind(event)
        .bind(user)
        .bind(actor)
        .bind(detail)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// The live principal row for a normalized email (soft-delete-aware).
    async fn live_user_by_email(
        &self,
        email: &str,
    ) -> Result<Option<(Uuid, String)>, PortalError> {
        let row = sqlx::query_as::<_, (Uuid, String)>(
            r#"SELECT id, status::text FROM portal.portal_users
               WHERE email = $1 AND (metadata->>'deleted_at') IS NULL"#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }

    /// The shared credential check + bearer mint after the throttle and
    /// status gates: uniform refusal on every failing arm.
    async fn verify_and_mint(
        &self,
        email: &str,
        presented: &str,
        ip: &str,
        audit_event_ok: &str,
    ) -> Result<(Uuid, String), PortalError> {
        let id_key = Self::identity_key(email);
        let ip_key = Self::ip_key(ip);
        let now = Utc::now();
        self.attempts.check(&id_key, now)?;
        self.attempts.check(&ip_key, now)?;

        let Some(user) = self.live_user_by_email(email).await? else {
            // Unknown email — SAME body as a wrong credential, and both
            // Tier B counters still bite (the book does not reveal which
            // arm refused).
            self.attempts.register_failure(&id_key, Utc::now());
            self.attempts.register_failure(&ip_key, Utc::now());
            self.audit(
                "login_refused",
                None,
                "anonymous",
                serde_json::json!({ "email_domain": email.split('@').nth(1) }),
            )
            .await?;
            return Err(PortalError::InvalidCredentials);
        };
        let (user_id, status) = user;
        if status != "active" {
            // Revoked/archived/invited — uniform refusal, no status oracle.
            self.attempts.register_failure(&id_key, Utc::now());
            self.attempts.register_failure(&ip_key, Utc::now());
            self.audit("login_refused", Some(user_id), "anonymous", serde_json::json!({}))
                .await?;
            return Err(PortalError::InvalidCredentials);
        }

        // The credential port — fail-closed when unwired.
        match self.credentials.check(CredentialCheck { email, presented }).await {
            Some(Ok(true)) => {}
            Some(Ok(false)) => {
                self.attempts.register_failure(&id_key, Utc::now());
                self.attempts.register_failure(&ip_key, Utc::now());
                self.audit("login_refused", Some(user_id), "anonymous", serde_json::json!({}))
                    .await?;
                return Err(PortalError::InvalidCredentials);
            }
            Some(Err(e)) => return Err(PortalError::Internal(format!("verifier: {e}"))),
            None => {
                self.audit(
                    "credential_port_not_composed",
                    Some(user_id),
                    "system",
                    serde_json::json!({ "verb": "login" }),
                )
                .await?;
                return Err(PortalError::CredentialPortNotComposed);
            }
        }

        // Success: reset BOTH Tier B counters, touch last_login, mint.
        self.attempts.reset(&id_key);
        self.attempts.reset(&ip_key);
        let _ = sqlx::query("UPDATE portal.portal_users SET last_login_at = NOW() WHERE id = $1")
            .bind(user_id)
            .execute(&self.pool)
            .await;
        let bearer = self.tokens.mint_bearer(user_id, DEFAULT_BEARER_TTL_HOURS).await?;
        self.audit(audit_event_ok, Some(user_id), "recipient", serde_json::json!({}))
            .await?;
        Ok((user_id, bearer))
    }

    /// Login (public, throttled): uniform refusal on every failing arm;
    /// success returns the principal id + a fresh bearer.
    pub async fn login(&self, email: &str, presented: &str, ip: &str) -> Result<(Uuid, String), PortalError> {
        let email = normalize_email(email);
        if email.is_empty() {
            return Err(PortalError::InvalidCredentials);
        }
        self.verify_and_mint(&email, presented, ip, "login_succeeded").await
    }

    /// Signup (public, throttled, POLICY-GATED): the kill switch is
    /// re-read on every call; closed (the default) refuses with the
    /// typed SignupClosed + an audited refusal row; open runs the same
    /// credential discipline as login against an existing or
    /// to-be-created principal.
    pub async fn signup(
        &self,
        email: &str,
        presented: &str,
        ip: &str,
    ) -> Result<(Uuid, String), PortalError> {
        let email = normalize_email(email);
        if email.is_empty() || !email.contains('@') {
            return Err(PortalError::InvalidInput("a valid email is required".into()));
        }
        if !self.policy.signup_open().await {
            self.audit(
                "signup_refused_policy_closed",
                None,
                "anonymous",
                serde_json::json!({}),
            )
            .await?;
            return Err(PortalError::SignupClosed);
        }
        match self.live_user_by_email(&email).await? {
            Some((_existing_id, status)) => {
                if status != "active" {
                    return Err(PortalError::InvalidCredentials);
                }
                // An existing principal "signing up" is just a login.
                self.verify_and_mint(&email, presented, ip, "login_succeeded").await
            }
            None => {
                // Policy open + credential verified -> create the principal.
                let id_key = Self::identity_key(&email);
                let ip_key = Self::ip_key(ip);
                self.attempts.check(&id_key, Utc::now())?;
                self.attempts.check(&ip_key, Utc::now())?;
                match self.credentials.check(CredentialCheck { email: &email, presented }).await {
                    Some(Ok(true)) => {}
                    Some(Ok(false)) => {
                        self.attempts.register_failure(&id_key, Utc::now());
                        self.attempts.register_failure(&ip_key, Utc::now());
                        return Err(PortalError::InvalidCredentials);
                    }
                    Some(Err(e)) => return Err(PortalError::Internal(format!("verifier: {e}"))),
                    None => {
                        self.audit(
                            "credential_port_not_composed",
                            None,
                            "system",
                            serde_json::json!({ "verb": "signup" }),
                        )
                        .await?;
                        return Err(PortalError::CredentialPortNotComposed);
                    }
                }
                let user_id = Uuid::new_v4();
                let inserted = sqlx::query_scalar::<_, Uuid>(
                    r#"INSERT INTO portal.portal_users (id, email, status)
                       VALUES ($1, $2, 'active')
                       ON CONFLICT DO NOTHING
                       RETURNING id"#,
                )
                .bind(user_id)
                .bind(&email)
                .fetch_optional(&self.pool)
                .await?;
                let user_id = match inserted {
                    Some(id) => id,
                    // The unique-on-live email index lost the race (PI-07):
                    // fall back to the winner — the principal exists once.
                    None => match self.live_user_by_email(&email).await? {
                        Some((id, _)) => id,
                        None => {
                            return Err(PortalError::Internal(
                                "signup race: principal missing after conflict".into(),
                            ))
                        }
                    },
                };
                self.audit(
                    "signup_created",
                    Some(user_id),
                    "recipient",
                    serde_json::json!({}),
                )
                .await?;
                let bearer = self.tokens.mint_bearer(user_id, DEFAULT_BEARER_TTL_HOURS).await?;
                Ok((user_id, bearer))
            }
        }
    }

    /// Revoke a principal's access (officer verb / lifecycle arm):
    /// status -> revoked (stamped, never cleared), every live bearer
    /// revoked, every pending invitation for their email revoked.
    /// CONDITIONAL on the current status: a principal already revoked
    /// (e.g. a replayed lifecycle envelope) flips nothing and returns
    /// `false` — the at-least-once bus can redeliver freely; every
    /// downstream audit row fires only on the performed edge. Audited
    /// as `lifecycle_access_revoked` when driven by the subscription,
    /// `bearer_revoked`/`invite_revoked` by the sweeps themselves.
    pub async fn revoke_access(&self, user_id: Uuid, reason: &str, actor: &str) -> Result<bool, PortalError> {
        let email = sqlx::query_scalar::<_, String>(
            r#"UPDATE portal.portal_users
               SET status = 'revoked', revoked_at = NOW(), revocation_reason = $2
               WHERE id = $1 AND status <> 'revoked'
                 AND (metadata->>'deleted_at') IS NULL
               RETURNING email"#,
        )
        .bind(user_id)
        .bind(reason)
        .fetch_optional(&self.pool)
        .await?;
        let Some(email) = email else {
            // Already revoked (replay) or absent: nothing to do. The
            // bearer/invite sweeps below would find nothing live anyway.
            return Ok(false);
        };

        self.tokens.revoke_all_bearers(user_id, reason, actor).await?;
        self.revoke_pending_invites_silent(&email, reason, actor).await?;
        Ok(true)
    }

    /// The invite-revocation arm without a second audit actor label
    /// (keeps one audit row per edge — the count rides the detail).
    async fn revoke_pending_invites_silent(&self, email: &str, reason: &str, actor: &str) -> Result<(), PortalError> {
        sqlx::query(
            r#"UPDATE portal.portal_invites
               SET status = 'revoked', revoked_at = NOW(), revocation_reason = $2
               WHERE recipient_email = $1 AND status = 'pending'"#,
        )
        .bind(email)
        .bind(reason)
        .execute(&self.pool)
        .await?;
        let _ = actor;
        Ok(())
    }
}

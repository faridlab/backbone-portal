//! `InviteService` — the invitation capability lifecycle (hand-written;
//! user-owned; see `metaphor.codegen.yaml`).
//!
//! Invitation-only is the SHIPPED access path (the signup policy default
//! OFF, PI-09/PI-18). An invitation IS a Tier A capability (ADR-0018):
//! the emailed link carries `{invite_id}.{nonce}.{exp}.{mac}`, the MAC
//! HMAC-SHA256 over `(id, nonce, grant, exp)` with the grant
//! `portal_invite:{recipient_email}` — the recipient-binding makes a
//! forwarded link refuse at redemption (the email presenting the link
//! must agree with the email the capability was minted for).
//!
//! The revocation list is EXPLICIT and row-level: `status -> revoked`
//! plus `revoked_at`/`revocation_reason` (officer verb, and the
//! archive/delete/login-kill trigger family on the linked principal).
//! Redemption is ONE conditional UPDATE (`pending -> redeemed`) — the
//! PI-07 grant-race fix: the database is the fence, not a check-then-
//! insert search, and a second redemption attempt finds nothing to flip.
//! Every mint, redemption, and revocation is a durable audit row.

use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::application::service::portal_error::PortalError;
use crate::application::service::token_service::{
    parse_capability, TokenService, DEFAULT_INVITE_TTL_DAYS,
};

/// Normalize an email to the comparison form (trim + lowercase) — every
/// lookup, mint, and grant binds to this form.
pub fn normalize_email(email: &str) -> String {
    email.trim().to_lowercase()
}

/// One invitation row (the service's own read shape).
#[derive(Debug, Clone)]
pub struct InviteRow {
    pub id: Uuid,
    pub recipient_email: String,
    pub nonce: String,
    pub expires_at: DateTime<Utc>,
    pub status: String,
}

/// The invitation engine: mint, redeem (one-shot), revoke, sweep.
pub struct InviteService {
    pool: PgPool,
    tokens: std::sync::Arc<TokenService>,
}

impl InviteService {
    pub fn new(pool: PgPool, tokens: std::sync::Arc<TokenService>) -> Self {
        Self { pool, tokens }
    }

    async fn audit(
        &self,
        event: &str,
        user: Option<Uuid>,
        invite: Option<Uuid>,
        actor: &str,
        detail: serde_json::Value,
    ) -> Result<(), PortalError> {
        sqlx::query(
            r#"INSERT INTO portal.portal_audit_log
                 (id, event, portal_user_id, invite_id, actor, detail)
               VALUES ($1, $2::portal_audit_event, $3, $4, $5, $6)"#,
        )
        .bind(Uuid::new_v4())
        .bind(event)
        .bind(user)
        .bind(invite)
        .bind(actor)
        .bind(detail)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// Mint an invitation for `recipient_email` (officer verb). Returns
    /// the invitation link — the ONLY thing that authorizes redemption.
    /// Audited (`invite_minted`).
    pub async fn mint_invite(
        &self,
        recipient_email: &str,
        granted_by: Option<Uuid>,
        ttl_days: i64,
    ) -> Result<String, PortalError> {
        let email = normalize_email(recipient_email);
        if !email.contains('@') || email.len() > 320 {
            return Err(PortalError::InvalidInput("a valid recipient email is required".into()));
        }
        let id = Uuid::new_v4();
        let nonce = crate::application::service::token_service::mint_nonce();
        let ttl = if ttl_days >= 1 { ttl_days } else { DEFAULT_INVITE_TTL_DAYS };
        let expires = Utc::now() + Duration::days(ttl);
        sqlx::query(
            r#"INSERT INTO portal.portal_invites
                 (id, recipient_email, token_nonce, token_expires_at, status, granted_by)
               VALUES ($1, $2, $3, $4, 'pending', $5)"#,
        )
        .bind(id)
        .bind(&email)
        .bind(&nonce)
        .bind(expires)
        .bind(granted_by)
        .execute(&self.pool)
        .await?;
        self.audit(
            "invite_minted",
            None,
            Some(id),
            granted_by.map(|u| u.to_string()).as_deref().unwrap_or("system"),
            serde_json::json!({ "recipient_email": email, "expires_at": expires.to_rfc3339() }),
        )
        .await?;
        self.tokens.invite_link(id, &nonce, &email, expires.timestamp())
    }

    /// Redeem an invitation — the ONE-SHOT access verb. The presenting
    /// email must agree with the capability's grant (forwarded links
    /// refuse), the row must be live + pending + unexpired, and the flip
    /// `pending -> redeemed` is a single conditional UPDATE (the PI-07
    /// race fence — two concurrent redemptions cannot both win). On
    /// success: the principal row is created (`status = active`) and the
    /// FIRST bearer is minted. Every refusal — unknown, forged, expired,
    /// already-redeemed, revoked, email mismatch — shares ONE body
    /// ([`PortalError::CredentialRefused`], no oracle). Audited on both
    /// edges (`invite_redeemed`).
    pub async fn redeem(
        &self,
        link: &str,
        presenting_email: &str,
    ) -> Result<(Uuid, String), PortalError> {
        let presenting = normalize_email(presenting_email);
        let parts = parse_capability(link).ok_or(PortalError::CredentialRefused)?;

        let row = sqlx::query_as::<_, (String, String, DateTime<Utc>, String)>(
            r#"SELECT recipient_email, token_nonce, token_expires_at, status::text
               FROM portal.portal_invites
               WHERE id = $1 AND (metadata->>'deleted_at') IS NULL"#,
        )
        .bind(parts.id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(PortalError::CredentialRefused)?;
        let (recipient_email, nonce, expires_at, status) = row;

        if !self.tokens.verify_invite_mac(
            parts.id,
            &nonce,
            &recipient_email,
            expires_at.timestamp(),
            &parts.mac,
        ) || parts.nonce != nonce
            || parts.exp != expires_at.timestamp()
            || presenting != recipient_email
            || status != "pending"
            || Utc::now() > expires_at
        {
            tracing::warn!(invite = %parts.id, "portal_invite_verify_failed");
            return Err(PortalError::CredentialRefused);
        }

        let user_id = Uuid::new_v4();
        let mut tx = self.pool.begin().await?;
        // THE one-shot fence: only a pending row flips; a racing second
        // redemption finds zero rows and refuses.
        let redeemed = sqlx::query(
            r#"UPDATE portal.portal_invites
               SET status = 'redeemed', redeemed_by = $2, redeemed_at = NOW()
               WHERE id = $1 AND status = 'pending'"#,
        )
        .bind(parts.id)
        .bind(user_id)
        .execute(&mut *tx)
        .await?
        .rows_affected();
        if redeemed != 1 {
            return Err(PortalError::CredentialRefused);
        }
        sqlx::query(
            r#"INSERT INTO portal.portal_users (id, email, status)
               VALUES ($1, $2, 'active')"#,
        )
        .bind(user_id)
        .bind(&recipient_email)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;

        self.audit(
            "invite_redeemed",
            Some(user_id),
            Some(parts.id),
            "recipient",
            serde_json::json!({ "email": recipient_email }),
        )
        .await?;

        let bearer = self
            .tokens
            .mint_bearer(user_id, crate::application::service::token_service::DEFAULT_BEARER_TTL_HOURS)
            .await?;
        Ok((user_id, bearer))
    }

    /// Revoke one invitation (officer verb). Audited (`invite_revoked`).
    pub async fn revoke_invite(&self, invite_id: Uuid, reason: &str, actor: &str) -> Result<(), PortalError> {
        let n = sqlx::query(
            r#"UPDATE portal.portal_invites
               SET status = 'revoked', revoked_at = NOW(), revocation_reason = $2
               WHERE id = $1 AND status = 'pending'"#,
        )
        .bind(invite_id)
        .bind(reason)
        .execute(&self.pool)
        .await?
        .rows_affected();
        if n != 1 {
            return Err(PortalError::NotFound(format!("pending invite {invite_id}")));
        }
        self.audit(
            "invite_revoked",
            None,
            Some(invite_id),
            actor,
            serde_json::json!({ "reason": reason }),
        )
        .await
    }

    /// Revoke every pending invitation for an email — the trigger arm of
    /// the explicit revocation list (archive/delete/login-kill on a
    /// principal kills their outstanding invitations too). Audited with
    /// the count.
    pub async fn revoke_pending_for_email(&self, email: &str, reason: &str, actor: &str) -> Result<u64, PortalError> {
        let email = normalize_email(email);
        let n = sqlx::query(
            r#"UPDATE portal.portal_invites
               SET status = 'revoked', revoked_at = NOW(), revocation_reason = $2
               WHERE recipient_email = $1 AND status = 'pending'"#,
        )
        .bind(&email)
        .bind(reason)
        .execute(&self.pool)
        .await?
        .rows_affected();
        if n > 0 {
            self.audit(
                "invite_revoked",
                None,
                None,
                actor,
                serde_json::json!({ "reason": reason, "email": email, "revoked_count": n }),
            )
            .await?;
        }
        Ok(n)
    }

    /// Mark past-expiry pending invitations `expired` (the sweep verb —
    /// safety never depends on it: expired links refuse at verify).
    pub async fn sweep_expired(&self) -> Result<u64, PortalError> {
        let n = sqlx::query(
            r#"UPDATE portal.portal_invites
               SET status = 'expired'
               WHERE status = 'pending' AND token_expires_at < NOW()"#,
        )
        .execute(&self.pool)
        .await?
        .rows_affected();
        Ok(n)
    }
}

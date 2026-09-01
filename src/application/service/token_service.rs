//! `TokenService` — the TWO independent Tier A credentials of the PI-01
//! split (hand-written; user-owned; see `metaphor.codegen.yaml`).
//!
//! ADR-0018 re-declared on the portal shapes:
//!
//! - **Credential one — the expiring rotatable BEARER** (the survey
//!   attempt machinery, adapted to a principal session): the presented
//!   credential is `{token_id}.{nonce}.{exp}.{mac}`; the MAC is
//!   HMAC-SHA256 over `(id, nonce, grant, exp)` with the grant
//!   `portal_bearer:{user_id}`, keyed by `PORTAL_BEARER_TOKEN_SECRET`
//!   with the `portal-bearer` domain separator. The MAC is recomputed
//!   over the STORED row fields at verify, so a forged nonce/exp cannot
//!   ride a valid id. MULTI-USE within its life; rotation mints a fresh
//!   nonce + expiry atomically and the old nonce dies with the UPDATE.
//!   The principal's status is checked at EVERY verification — a revoked
//!   or archived principal's live-looking tokens refuse.
//! - **Credential two — the per-recipient HMAC** (the digest unsubscribe
//!   machinery): `{user_id}.{email}.{exp}.{mac}` with the MAC over
//!   `(user_id, email, exp)` under the `portal-recipient` domain
//!   separator, keyed by its OWN secret (`PORTAL_RECIPIENT_TOKEN_SECRET`)
//!   and STATELESS (no row — the mint audit row is its only durable
//!   trace). Its input NEVER covers the bearer: rotating the bearer (or
//!   re-minting the recipient link) leaves the other credential
//!   untouched — the independence PI-01 demands.
//! - **Mint audit** (the W7-C4 requirement none of the composed
//!   precedents carries): every mint, rotation, and revocation of either
//!   credential lands as a durable `portal.portal_audit_log` row.
//! - **Tier B** (ADR-0018): every verification endpoint is throttled per
//!   identity AND per IP with an escalating lockout (the survey
//!   session-code book); the pure curve is probe-asserted.
//!
//! Both secrets are mandatory at mint time: an unset secret is a typed
//! loud failure, never a zero-secret fallback that would let anyone
//! forge credentials.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use chrono::{DateTime, Duration, Utc};
use hmac::{Hmac, Mac};
use rand::RngCore;
use sha2::Sha256;
use sqlx::PgPool;
use uuid::Uuid;

use crate::application::service::portal_error::PortalError;
use crate::domain::entity::{PortalUser, PortalUserStatus};

/// Env var holding the HMAC secret for BEARER credentials (and the
/// invitation capabilities — same key, distinct domain separators).
pub const PORTAL_BEARER_SECRET_ENV: &str = "PORTAL_BEARER_TOKEN_SECRET";

/// Env var holding the HMAC secret for the per-recipient credential. A
/// SEPARATE secret is the honest reading of "two independent
/// credentials": rotating one secret never invalidates the other family.
pub const PORTAL_RECIPIENT_SECRET_ENV: &str = "PORTAL_RECIPIENT_TOKEN_SECRET";

/// Default bearer lifetime at mint (24 h; rotation re-stamps it).
pub const DEFAULT_BEARER_TTL_HOURS: i64 = 24;

/// Default invitation capability lifetime at mint (14 d).
pub const DEFAULT_INVITE_TTL_DAYS: i64 = 14;

/// Per-recipient HMAC lifetime (180 d — the digest precedent: a link
/// comfortably outlives the mailing cadence while a leaked one
/// eventually dies).
pub const RECIPIENT_HMAC_TTL_DAYS: i64 = 180;

/// Domain separators — keep the two grant families distinct under the
/// shared bearer key, and the recipient family distinct by construction.
const DOMAIN_BEARER: &str = "portal-bearer";
const DOMAIN_INVITE: &str = "portal-invite";
const DOMAIN_RECIPIENT: &str = "portal-recipient";

type HmacSha256 = Hmac<Sha256>;

// ─── Tier B policy knobs (probe-asserted as pure functions) ───────────────────

/// Consecutive failures before the first lockout kicks in.
pub const ATTEMPT_MAX_FAILURES: i32 = 5;
/// First lockout duration; doubles per extra failure.
pub const ATTEMPT_LOCK_BASE_SECONDS: i64 = 30;
/// Ceiling for the escalating lockout (15 minutes).
pub const ATTEMPT_LOCK_CAP_SECONDS: i64 = 900;
/// Minimum spacing between attempts (anti-hammering).
pub const ATTEMPT_SPACING: Duration = Duration::seconds(1);

/// Escalating lockout (the survey pure curve): `failures` have now
/// accumulated (pass the post-increment count). Lock starts at
/// [`ATTEMPT_MAX_FAILURES`], doubles per extra failure, caps at
/// [`ATTEMPT_LOCK_CAP_SECONDS`]. `None` = not locked yet.
pub fn lockout_until(failures: i32, now: DateTime<Utc>) -> Option<DateTime<Utc>> {
    if failures < ATTEMPT_MAX_FAILURES {
        return None;
    }
    let doubles = (failures - ATTEMPT_MAX_FAILURES) as u32;
    let secs = ATTEMPT_LOCK_BASE_SECONDS.saturating_mul(1i64 << doubles.min(16));
    Some(now + Duration::seconds(secs.min(ATTEMPT_LOCK_CAP_SECONDS)))
}

/// One key's failure state.
#[derive(Debug, Clone)]
struct FailureEntry {
    failures: i32,
    locked_until: Option<DateTime<Utc>>,
    last_attempt: Option<DateTime<Utc>>,
}

/// The in-memory Tier B attempt book: per-identity AND per-IP counters
/// (`portal|id:{identity}` / `portal|ip:{ip}`). Pure bookkeeping — the
/// escalation curve lives in the pure [`lockout_until`]. In-memory per
/// composing service (the family trade: a multi-instance host fronts it
/// with a shared limiter).
#[derive(Debug, Default)]
pub struct AttemptBook {
    entries: Mutex<HashMap<String, FailureEntry>>,
}

impl AttemptBook {
    pub fn new() -> Self {
        Self::default()
    }

    /// Is this key currently locked out (or spacing-gated)? Returns the
    /// typed refusal to surface, if any.
    pub fn check(&self, key: &str, now: DateTime<Utc>) -> Result<(), PortalError> {
        let guard = self.entries.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(entry) = guard.get(key) {
            if let Some(until) = entry.locked_until {
                if now < until {
                    return Err(PortalError::RateLimited {
                        retry_after_seconds: (until - now).num_seconds().max(1),
                    });
                }
            }
            if let Some(last) = entry.last_attempt {
                if now.signed_duration_since(last) < ATTEMPT_SPACING {
                    return Err(PortalError::RateLimited { retry_after_seconds: 1 });
                }
            }
        }
        Ok(())
    }

    /// Register a failure: increment, recompute the lockout, stamp the
    /// attempt time.
    pub fn register_failure(&self, key: &str, now: DateTime<Utc>) {
        let mut guard = self.entries.lock().unwrap_or_else(|e| e.into_inner());
        let entry = guard.entry(key.to_string()).or_insert(FailureEntry {
            failures: 0,
            locked_until: None,
            last_attempt: None,
        });
        entry.failures += 1;
        entry.locked_until = lockout_until(entry.failures, now);
        entry.last_attempt = Some(now);
    }

    /// Success resets the counter (the identity proved itself).
    pub fn reset(&self, key: &str) {
        self.entries
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .remove(key);
    }

    /// The recorded failure count (probe visibility).
    pub fn failures(&self, key: &str) -> i32 {
        self.entries
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .get(key)
            .map(|e| e.failures)
            .unwrap_or(0)
    }
}

// ─── the capability link machinery (shared by bearer + invite) ────────────────

/// The parsed `{id}.{nonce}.{exp}.{mac}` capability (bearer or invite —
/// the grant disambiguates at verify).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCapability {
    pub id: Uuid,
    pub nonce: String,
    pub exp: i64,
    pub mac: String,
}

/// Parse `{id}.{nonce}.{exp}.{mac}` — anything else is malformed (and
/// malformed is refusal-shaped, never a panic).
pub fn parse_capability(link: &str) -> Option<ParsedCapability> {
    let mut parts = link.split('.');
    let id = Uuid::parse_str(parts.next()?).ok()?;
    let nonce = parts.next()?.to_string();
    if nonce.is_empty() || !nonce.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let exp: i64 = parts.next()?.parse().ok()?;
    let mac = parts.next()?.to_string();
    if mac.len() != 64 || !mac.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    if parts.next().is_some() {
        return None;
    }
    Some(ParsedCapability { id, nonce, exp, mac })
}

/// A fresh 128-bit hex selector.
pub fn mint_nonce() -> String {
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// The MAC input: `(domain, id, nonce, grant, exp)` — the domain keeps
/// the bearer and invitation grant families distinct under one key.
fn capability_mac_input(domain: &str, id: &Uuid, nonce: &str, grant: &str, exp: i64) -> String {
    format!("{domain}|{id}.{nonce}.{grant}.{exp}")
}

fn compute_mac(secret: &[u8], input: &str) -> Result<String, PortalError> {
    let mut mac = HmacSha256::new_from_slice(secret)
        .map_err(|e| PortalError::Internal(format!("hmac init: {e}")))?;
    mac.update(input.as_bytes());
    Ok(mac
        .finalize()
        .into_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect())
}

/// Constant-time MAC comparison (`hmac`'s `verify_slice` — the `consteq`
/// requirement). A malformed candidate or an internal HMAC failure is a
/// refusal (fail-closed), never `false`-vs-error distinguishable.
fn verify_mac(secret: &[u8], input: &str, provided: &str) -> bool {
    let Ok(mut mac) = HmacSha256::new_from_slice(secret) else {
        return false;
    };
    mac.update(input.as_bytes());
    match hex_decode(provided) {
        Some(bytes) => mac.verify_slice(&bytes).is_ok(),
        None => false,
    }
}

fn hex_decode(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 {
        return None;
    }
    let mut out = Vec::with_capacity(s.len() / 2);
    let bytes = s.as_bytes();
    for pair in bytes.chunks(2) {
        let hi = (pair[0] as char).to_digit(16)?;
        let lo = (pair[1] as char).to_digit(16)?;
        out.push((hi * 16 + lo) as u8);
    }
    Some(out)
}

/// The public recipient credential: `{user_id}.{email}.{exp}.{mac_hex}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipientCredential {
    pub user_id: Uuid,
    pub email: String,
    pub exp: i64,
    pub mac: String,
}

fn recipient_mac_input(user_id: &Uuid, email: &str, exp: i64) -> String {
    format!("{DOMAIN_RECIPIENT}|{user_id}|{email}|{exp}")
}

// ─── the row shapes the service reads/writes ──────────────────────────────────

/// One live bearer row (the service's own read shape over
/// `portal.portal_tokens` — never the plaintext credential; the MAC is
/// recomputed from the row fields at verify).
#[derive(Debug, Clone)]
pub struct BearerRow {
    pub id: Uuid,
    pub user_id: Uuid,
    pub nonce: String,
    pub expires_at: DateTime<Utc>,
    pub status: String,
}

/// The verified principal context a bearer verification yields — the
/// session identity every read model and verb stamps from.
#[derive(Debug, Clone, PartialEq)]
pub struct PortalPrincipal {
    pub user_id: Uuid,
    pub email: String,
    pub display_name: Option<String>,
}

// ─── the service ──────────────────────────────────────────────────────────────

/// The credential engine: bearer mint/verify/rotate/revoke + the
/// stateless per-recipient HMAC + mint audit + the Tier B attempt book.
pub struct TokenService {
    pool: PgPool,
    bearer_secret: Vec<u8>,
    recipient_secret: Vec<u8>,
    attempts: Arc<AttemptBook>,
}

impl TokenService {
    /// Construct with explicit secrets (composition + probes).
    pub fn with_secrets(pool: PgPool, bearer_secret: &[u8], recipient_secret: &[u8]) -> Self {
        Self {
            pool,
            bearer_secret: bearer_secret.to_vec(),
            recipient_secret: recipient_secret.to_vec(),
            attempts: Arc::new(AttemptBook::new()),
        }
    }

    /// Construct from the environment; an unset secret is EMPTY here and
    /// every mint/verify that needs it fails loudly with the typed
    /// secret-not-configured error (no zero-secret fallback).
    pub fn from_env(pool: PgPool) -> Self {
        let bearer = std::env::var(PORTAL_BEARER_SECRET_ENV).unwrap_or_default();
        let recipient = std::env::var(PORTAL_RECIPIENT_SECRET_ENV).unwrap_or_default();
        Self::with_secrets(pool, bearer.as_bytes(), recipient.as_bytes())
    }

    pub fn attempt_book(&self) -> Arc<AttemptBook> {
        self.attempts.clone()
    }

    fn bearer_secret(&self) -> Result<&[u8], PortalError> {
        if self.bearer_secret.is_empty() {
            return Err(PortalError::BearerSecretNotConfigured);
        }
        Ok(&self.bearer_secret)
    }

    fn recipient_secret(&self) -> Result<&[u8], PortalError> {
        if self.recipient_secret.is_empty() {
            return Err(PortalError::RecipientSecretNotConfigured);
        }
        Ok(&self.recipient_secret)
    }

    async fn audit(
        &self,
        event: &str,
        user: Option<Uuid>,
        token: Option<Uuid>,
        actor: &str,
        detail: serde_json::Value,
    ) -> Result<(), PortalError> {
        sqlx::query(
            r#"INSERT INTO portal.portal_audit_log
                 (id, event, portal_user_id, token_id, actor, detail)
               VALUES ($1, $2::portal_audit_event, $3, $4, $5, $6)"#,
        )
        .bind(Uuid::new_v4())
        .bind(event)
        .bind(user)
        .bind(token)
        .bind(actor)
        .bind(detail)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // ── credential one: the BEARER ───────────────────────────────────────────

    fn bearer_grant(user_id: &Uuid) -> String {
        format!("portal_bearer:{user_id}")
    }

    /// Render the public credential string for a row.
    fn bearer_link(&self, row: &BearerRow) -> Result<String, PortalError> {
        let secret = self.bearer_secret()?;
        let exp = row.expires_at.timestamp();
        let mac = compute_mac(
            secret,
            &capability_mac_input(DOMAIN_BEARER, &row.id, &row.nonce, &Self::bearer_grant(&row.user_id), exp),
        )?;
        Ok(format!("{}.{}.{}.{}", row.id, row.nonce, exp, mac))
    }

    /// Mint a fresh bearer for a principal (redemption / signup / login).
    /// Audited (`bearer_minted`). The nonce is a fresh selector — never
    /// derived from any prior credential.
    pub async fn mint_bearer(&self, user_id: Uuid, ttl_hours: i64) -> Result<String, PortalError> {
        let _ = self.bearer_secret()?;
        let id = Uuid::new_v4();
        let nonce = mint_nonce();
        let expires = Utc::now() + Duration::hours(ttl_hours.max(1));
        sqlx::query(
            r#"INSERT INTO portal.portal_tokens
                 (id, user_id, token_nonce, token_expires_at, status)
               VALUES ($1, $2, $3, $4, 'active')"#,
        )
        .bind(id)
        .bind(user_id)
        .bind(&nonce)
        .bind(expires)
        .execute(&self.pool)
        .await?;
        self.audit(
            "bearer_minted",
            Some(user_id),
            Some(id),
            "system",
            serde_json::json!({ "expires_at": expires.to_rfc3339() }),
        )
        .await?;
        let row = BearerRow { id, user_id, nonce, expires_at: expires, status: "active".into() };
        self.bearer_link(&row)
    }

    /// Full verification: parse → load row + principal → recompute the
    /// MAC over the STORED fields → status/expiry gates. Unknown,
    /// malformed, forged, expired, rotated, revoked, and dead-principal
    /// share ONE refusal ([`PortalError::CredentialRefused`] — no
    /// oracle). The Tier B book is keyed by the ROUTE (identity + IP).
    pub async fn verify_bearer(&self, link: &str) -> Result<PortalPrincipal, PortalError> {
        let secret = self.bearer_secret()?;
        let parts = parse_capability(link).ok_or(PortalError::CredentialRefused)?;

        let row = sqlx::query_as::<_, (String, DateTime<Utc>, String, Uuid, String, Option<String>)>(
            r#"SELECT t.token_nonce, t.token_expires_at, t.status::text,
                      u.id, u.email, u.display_name
               FROM portal.portal_tokens t
               JOIN portal.portal_users u ON u.id = t.user_id
               WHERE t.id = $1
                 AND (t.metadata->>'deleted_at') IS NULL
                 AND (u.metadata->>'deleted_at') IS NULL"#,
        )
        .bind(parts.id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(PortalError::CredentialRefused)?;
        let (nonce, expires_at, token_status, user_id, email, display_name) = row;

        let grant = Self::bearer_grant(&user_id);
        let expected = capability_mac_input(DOMAIN_BEARER, &parts.id, &nonce, &grant, expires_at.timestamp());
        if !verify_mac(secret, &expected, &parts.mac)
            || parts.nonce != nonce
            || parts.exp != expires_at.timestamp()
        {
            tracing::warn!(token = %parts.id, "portal_bearer_verify_failed");
            return Err(PortalError::CredentialRefused);
        }
        if Utc::now() > expires_at {
            return Err(PortalError::CredentialRefused);
        }
        if token_status != "active" {
            return Err(PortalError::CredentialRefused);
        }
        // The principal's status gates EVERY verification — a revoked or
        // archived principal's live-looking tokens refuse.
        let user_status = sqlx::query_scalar::<_, String>(
            r#"SELECT status::text FROM portal.portal_users
               WHERE id = $1 AND (metadata->>'deleted_at') IS NULL"#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or(PortalError::CredentialRefused)?;
        if user_status != PortalUserStatus::Active.to_string() {
            return Err(PortalError::CredentialRefused);
        }

        // Best-effort last-used touch (observation only; gates nothing).
        let _ = sqlx::query("UPDATE portal.portal_tokens SET last_used_at = NOW() WHERE id = $1")
            .bind(parts.id)
            .execute(&self.pool)
            .await;

        Ok(PortalPrincipal { user_id, email, display_name })
    }

    /// The rotation verb: a VALID bearer is exchanged for a fresh one —
    /// new nonce + expiry on a new row, the old row flips to `rotated`
    /// atomically. The old credential dies with the UPDATE. Audited
    /// (`bearer_rotated`).
    pub async fn rotate_bearer(&self, link: &str, ttl_hours: i64) -> Result<String, PortalError> {
        let principal = self.verify_bearer(link).await?;
        let parts = parse_capability(link).ok_or(PortalError::CredentialRefused)?;

        let mut tx = self.pool.begin().await?;
        let new_id = Uuid::new_v4();
        let new_nonce = mint_nonce();
        let new_exp = Utc::now() + Duration::hours(ttl_hours.max(1));

        let rotated = sqlx::query(
            r#"UPDATE portal.portal_tokens
               SET status = 'rotated', rotated_at = NOW(), rotated_to = $2
               WHERE id = $1 AND status = 'active'"#,
        )
        .bind(parts.id)
        .bind(new_id)
        .execute(&mut *tx)
        .await?
        .rows_affected();
        if rotated != 1 {
            return Err(PortalError::CredentialRefused);
        }
        sqlx::query(
            r#"INSERT INTO portal.portal_tokens
                 (id, user_id, token_nonce, token_expires_at, status)
               VALUES ($1, $2, $3, $4, 'active')"#,
        )
        .bind(new_id)
        .bind(principal.user_id)
        .bind(&new_nonce)
        .bind(new_exp)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;

        self.audit(
            "bearer_rotated",
            Some(principal.user_id),
            Some(parts.id),
            "recipient",
            serde_json::json!({ "successor": new_id, "expires_at": new_exp.to_rfc3339() }),
        )
        .await?;

        let row = BearerRow {
            id: new_id,
            user_id: principal.user_id,
            nonce: new_nonce,
            expires_at: new_exp,
            status: "active".into(),
        };
        self.bearer_link(&row)
    }

    /// Revoke one bearer (officer verb / login-kill). Audited.
    pub async fn revoke_bearer(&self, token_id: Uuid, reason: &str, actor: &str) -> Result<(), PortalError> {
        let user_id = sqlx::query_scalar::<_, Uuid>(
            r#"UPDATE portal.portal_tokens
               SET status = 'revoked', revoked_at = NOW(), revocation_reason = $2
               WHERE id = $1 AND status = 'active'
               RETURNING user_id"#,
        )
        .bind(token_id)
        .bind(reason)
        .fetch_optional(&self.pool)
        .await?;
        match user_id {
            Some(uid) => {
                self.audit(
                    "bearer_revoked",
                    Some(uid),
                    Some(token_id),
                    actor,
                    serde_json::json!({ "reason": reason }),
                )
                .await
            }
            None => Err(PortalError::NotFound(format!("live bearer {token_id}"))),
        }
    }

    /// Revoke EVERY live bearer of a principal (the lifecycle / officer
    /// revocation sweep). Audited once with the count in the detail.
    pub async fn revoke_all_bearers(&self, user_id: Uuid, reason: &str, actor: &str) -> Result<u64, PortalError> {
        let n = sqlx::query(
            r#"UPDATE portal.portal_tokens
               SET status = 'revoked', revoked_at = NOW(), revocation_reason = $2
               WHERE user_id = $1 AND status = 'active'"#,
        )
        .bind(user_id)
        .bind(reason)
        .execute(&self.pool)
        .await?
        .rows_affected();
        if n > 0 {
            self.audit(
                "bearer_revoked",
                Some(user_id),
                None,
                actor,
                serde_json::json!({ "reason": reason, "revoked_count": n }),
            )
            .await?;
        }
        Ok(n)
    }

    // ── credential two: the per-recipient HMAC ───────────────────────────────

    /// Mint the per-recipient credential for (user, email) at `now`:
    /// `{user_id}.{email}.{exp}.{mac}` under the recipient secret.
    /// STATELESS by design (no row); the mint audit row is its durable
    /// trace. Rotating this credential = minting a fresh one (the old
    /// dies at its own expiry — the digest posture); it NEVER touches the
    /// bearer table.
    pub async fn mint_recipient_credential(
        &self,
        user_id: Uuid,
        email: &str,
    ) -> Result<String, PortalError> {
        let secret = self.recipient_secret()?;
        let exp_ts = (Utc::now() + Duration::days(RECIPIENT_HMAC_TTL_DAYS)).timestamp();
        let mac = compute_mac(secret, &recipient_mac_input(&user_id, email, exp_ts))?;
        self.audit(
            "recipient_hmac_minted",
            Some(user_id),
            None,
            "system",
            serde_json::json!({ "expires_at": exp_ts }),
        )
        .await?;
        Ok(format!("{user_id}.{email}.{exp_ts}.{mac}"))
    }

    /// Verify a per-recipient credential PURELY (no DB roundtrip — the
    /// capability is verifiable from the link alone, the ADR-0018 Tier A
    /// property). Constant-time MAC; expired or forged is a bare `false`.
    ///
    /// Parse order matters: the recipient email itself may contain dots,
    /// so the arms are consumed from BOTH ends — first the user id, then
    /// (from the right) the mac and the expiry — and everything left in
    /// the middle IS the email, rejoined. A link with a missing email
    /// arm leaves an empty middle and its MAC cannot match anything
    /// ever minted.
    pub fn verify_recipient_credential(&self, value: &str, now: i64) -> bool {
        let Some(secret) = self.recipient_secret().ok() else {
            return false;
        };
        let mut parts = value.split('.');
        let Ok(user_id) = Uuid::parse_str(parts.next().unwrap_or_default()) else {
            return false;
        };
        let mac = parts.next_back().unwrap_or_default().to_string();
        if mac.len() != 64 {
            return false;
        }
        let Ok(exp) = parts.next_back().unwrap_or_default().parse::<i64>() else {
            return false;
        };
        let email = parts.collect::<Vec<_>>().join(".");
        verify_mac(secret, &recipient_mac_input(&user_id, &email, exp), &mac) && now <= exp
    }

    // ── the invitation capability (grant family under the bearer key) ───────

    /// Render the invitation link's MAC arm — the invitation grant binds
    /// to the recipient email (`portal_invite:{email}`), so a link
    /// forwarded to another mailbox refuses at redemption.
    pub(crate) fn invite_link(
        &self,
        invite_id: Uuid,
        nonce: &str,
        recipient_email: &str,
        exp: i64,
    ) -> Result<String, PortalError> {
        let secret = self.bearer_secret()?;
        let grant = format!("portal_invite:{recipient_email}");
        let mac = compute_mac(
            secret,
            &capability_mac_input(DOMAIN_INVITE, &invite_id, nonce, &grant, exp),
        )?;
        Ok(format!("{invite_id}.{nonce}.{exp}.{mac}"))
    }

    /// Verify an invitation link against the STORED row fields (the
    /// redemption arm re-checks everything against the database; this is
    /// the MAC arm shared with it). Constant-time.
    pub(crate) fn verify_invite_mac(
        &self,
        invite_id: Uuid,
        stored_nonce: &str,
        recipient_email: &str,
        stored_exp: i64,
        provided_mac: &str,
    ) -> bool {
        let Some(secret) = self.bearer_secret().ok() else {
            return false;
        };
        let grant = format!("portal_invite:{recipient_email}");
        verify_mac(
            secret,
            &capability_mac_input(DOMAIN_INVITE, &invite_id, stored_nonce, &grant, stored_exp),
            provided_mac,
        )
    }

    // ── principal lookups (shared with the access/surface services) ─────────

    /// The live principal by id (soft-delete-aware).
    pub async fn live_principal(&self, user_id: Uuid) -> Result<Option<PortalUser>, PortalError> {
        let row = sqlx::query_as::<_, PortalUser>(
            r#"SELECT * FROM portal.portal_users
               WHERE id = $1 AND (metadata->>'deleted_at') IS NULL"#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }
}

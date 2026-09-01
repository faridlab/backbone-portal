//! `PolicyService` — the kill-switchable signup policy (hand-written;
//! user-owned; see `metaphor.codegen.yaml`).
//!
//! PI-09/PI-18: upstream flips b2c signup on at INSTALL time and the
//! website family carries DB-wide b2c installs — none of that ports. The
//! port ships open-signup as a declared policy surface that is OFF by
//! default and FAIL-CLOSED under every unreadable condition:
//!
//! - **no row** (the shipped state — zero install-time bootstrap, no
//!   seed migration inserts one) reads as CLOSED;
//! - **enabled = false** reads as CLOSED;
//! - only **enabled = true** (an officer's deliberate flip through the
//!   guarded verb, audited as `policy_changed`) opens self-service
//!   signup — and the officer can kill it again at any moment.
//!
//! The row is a singleton by a hand-hardening partial unique index (at
//! most one row EVER — history lives in the audit log, not in dead rows).

use sqlx::PgPool;
use uuid::Uuid;

use crate::application::service::portal_error::PortalError;

/// The signup policy engine: fail-closed read + guarded set.
pub struct PolicyService {
    pool: PgPool,
}

impl PolicyService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Is open signup currently allowed? FAIL-CLOSED: absent row, read
    /// error, or disabled row all read as `false`. This is the kill
    /// switch every signup attempt re-reads — a flip bites immediately.
    pub async fn signup_open(&self) -> bool {
        let Ok(enabled) = sqlx::query_scalar::<_, bool>(
            r#"SELECT enabled FROM portal.portal_signup_policies
               WHERE (metadata->>'deleted_at') IS NULL
               LIMIT 1"#,
        )
        .fetch_one(&self.pool)
        .await
        else {
            return false;
        };
        enabled
    }

    /// The guarded set verb (officer tree only): flips the switch and
    /// audits the flip (`policy_changed`). Idempotent — flipping to the
    /// current value still records the attempt.
    pub async fn set_signup_policy(
        &self,
        enabled: bool,
        note: Option<&str>,
        officer: Option<Uuid>,
    ) -> Result<(), PortalError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM portal.portal_signup_policies")
            .execute(&mut *tx)
            .await?;
        sqlx::query(
            r#"INSERT INTO portal.portal_signup_policies
                 (id, enabled, note, updated_by)
               VALUES ($1, $2, $3, $4)"#,
        )
        .bind(Uuid::new_v4())
        .bind(enabled)
        .bind(note)
        .bind(officer)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;

        sqlx::query(
            r#"INSERT INTO portal.portal_audit_log
                 (id, event, actor, detail)
               VALUES ($1, 'policy_changed'::portal_audit_event, $2, $3)"#,
        )
        .bind(Uuid::new_v4())
        .bind(officer.map(|u| u.to_string()).as_deref().unwrap_or("system"))
        .bind(serde_json::json!({ "enabled": enabled, "note": note }))
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

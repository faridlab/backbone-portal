//! `SapiensLifecycleHandler` — the sapiens lifecycle subscription
//! (hand-written; user-owned; see `metaphor.codegen.yaml`).
//!
//! The portal identity home is portal-owned, but the ACCOUNT lifecycle
//! lives in sapiens — so the portal subscribes to sapiens' integration
//! events on the HOST's bus (the digest `user_created_handler`
//! precedent: the host registers this handler at compose time with
//! `bus.register_handler(portal.lifecycle_handler()).await`; one
//! registration covers the in-proc path and the outbox-relay path).
//!
//! ## The subscribed contract (backbone-sapiens v0.2.5, verified
//! first-hand against the published tag)
//!
//! | event type | payload fields | portal arm |
//! |---|---|---|
//! | `sapiens.user.created` | `user_id`, `email`, username/names, occurred_at | onboarding: link an existing portal principal (`sapiens_user_id` stamp); no principal = audited `onboarding_no_invitation` — never a silent skip |
//! | `sapiens.user.deactivated` | `user_id`, reason?, occurred_at | revoke portal access + invalidate every live bearer |
//! | `sapiens.user.deleted` | `user_id`, occurred_at | same revocation arm (deletion is terminal) |
//! | `sapiens.user.anonymized` | `user_id`, occurred_at | same revocation arm (anonymization is terminal for the identity — every credential, session, and portal access keyed on the user must die) |
//!
//! The typed event structs this module codes against are the ones the
//! v0.2.5 tag actually publishes: `backbone_sapiens::infrastructure::
//! messaging::{UserCreatedIntegrationEvent, UserDeletedIntegrationEvent,
//! UserAnonymizedIntegrationEvent}` (the same surface the host's
//! integration probe consumes).
//!
//! ## The producers (live since sapiens v0.2.5)
//!
//! All four event types have live producers in sapiens v0.2.5: the
//! register + admin create paths publish `sapiens.user.created`, the
//! deactivation lifecycle outbox publishes `sapiens.user.deactivated`,
//! soft-delete publishes `sapiens.user.deleted`, and the anonymization
//! record publisher emits `sapiens.user.anonymized`. Before v0.2.5
//! only `created` had producers — the arms existed but never fired;
//! that interim posture is recorded in the sapiens O-2 threat notes.
//!
//! ## Idempotency (the at-least-once bus)
//!
//! Both arms are re-delivery safe: the onboarding stamp is a
//! conditional UPDATE (`sapiens_user_id IS NULL`), and the revocation
//! arm's status flip is conditional (`status <> 'revoked'`), so a
//! replayed envelope finds nothing left to do and the audit rows fire
//! only on the performed edge.

use async_trait::async_trait;
use backbone_messaging::{EventError, IntegrationEventEnvelope, IntegrationEventHandler};
use serde::Deserialize;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

use crate::application::service::access_service::AccessService;
use crate::application::service::invite_service::normalize_email;

const HANDLER: &str = "portal.sapiens_lifecycle";

/// The `sapiens.user.created` payload (the v0.2.4
/// `UserCreatedIntegrationEvent` wire shape — first-hand verified; only
/// the fields the portal arms read are declared).
#[derive(Debug, Clone, Deserialize)]
pub struct UserCreatedPayload {
    pub user_id: String,
    pub email: String,
}

/// The `sapiens.user.deactivated` / `sapiens.user.deleted` /
/// `sapiens.user.anonymized` payload (the v0.2.5
/// `UserDeletedIntegrationEvent` / `UserAnonymizedIntegrationEvent`
/// wire shape — user_id only).
#[derive(Debug, Clone, Deserialize)]
pub struct UserGonePayload {
    pub user_id: String,
}

/// The lifecycle consumer. Constructed by the module builder; registered
/// on the HOST's integration bus at compose time.
pub struct SapiensLifecycleHandler {
    pool: PgPool,
    access: Arc<AccessService>,
}

impl SapiensLifecycleHandler {
    pub fn new(pool: PgPool, access: Arc<AccessService>) -> Self {
        Self { pool, access }
    }

    /// The event types this handler subscribes to (exact match).
    pub const PATTERNS: [&'static str; 4] = [
        "sapiens.user.created",
        "sapiens.user.deactivated",
        "sapiens.user.deleted",
        "sapiens.user.anonymized",
    ];
}

/// The onboarding facts derived from the published typed event
/// `backbone_sapiens::infrastructure::messaging::
/// UserCreatedIntegrationEvent` (v0.2.4). The published struct is the
/// contract this module codes against; deriving the wire facts from it
/// (rather than hand-copying strings) keeps the dependency
/// load-bearing — if the export moves, this crate stops compiling
/// loudly instead of drifting.
pub fn created_facts_from_published(
    event: &backbone_sapiens::infrastructure::messaging::UserCreatedIntegrationEvent,
) -> UserCreatedPayload {
    UserCreatedPayload {
        user_id: event.user_id.clone(),
        email: event.email.clone(),
    }
}

/// The deletion facts derived from the published typed event
/// `backbone_sapiens::infrastructure::messaging::
/// UserDeletedIntegrationEvent` (v0.2.5).
pub fn deleted_facts_from_published(
    event: &backbone_sapiens::infrastructure::messaging::UserDeletedIntegrationEvent,
) -> UserGonePayload {
    UserGonePayload { user_id: event.user_id.clone() }
}

/// The anonymization facts derived from the published typed event
/// `backbone_sapiens::infrastructure::messaging::
/// UserAnonymizedIntegrationEvent` (v0.2.5). The struct exists only
/// from v0.2.5 — referencing it keeps the tag repoint load-bearing:
/// this crate stops compiling against any older sapiens pin.
pub fn anonymized_facts_from_published(
    event: &backbone_sapiens::infrastructure::messaging::UserAnonymizedIntegrationEvent,
) -> UserGonePayload {
    UserGonePayload { user_id: event.user_id.clone() }
}

impl SapiensLifecycleHandler {
    /// The onboarding arm (`sapiens.user.created`): stamp the lifecycle
    /// link on an existing portal principal (idempotent — only a NULL
    /// link flips); otherwise audited non-action.
    async fn on_created(&self, envelope: &IntegrationEventEnvelope) -> Result<(), EventError> {
        let payload: UserCreatedPayload = serde_json::from_value(envelope.payload.clone())
            .map_err(|e| {
                EventError::handler(
                    HANDLER,
                    format!("created payload: {e} (envelope {})", envelope.id),
                )
            })?;
        let Ok(sapiens_user) = Uuid::parse_str(&payload.user_id) else {
            return Err(EventError::handler(
                HANDLER,
                format!("created payload.user_id is not a uuid: {}", payload.user_id),
            ));
        };
        let email = normalize_email(&payload.email);

        // The conditional stamp: only a live, unlinked principal flips —
        // a replayed envelope finds the link already set and this
        // returns None (idempotent non-action, NOT an error).
        let linked = sqlx::query_scalar::<_, Uuid>(
            r#"UPDATE portal.portal_users
               SET sapiens_user_id = $1
               WHERE email = $2 AND sapiens_user_id IS NULL
                 AND (metadata->>'deleted_at') IS NULL
               RETURNING id"#,
        )
        .bind(sapiens_user)
        .bind(&email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| EventError::handler(HANDLER, format!("onboarding stamp: {e}")))?;

        match linked {
            Some(portal_user_id) => {
                sqlx::query(
                    r#"INSERT INTO portal.portal_audit_log
                         (id, event, portal_user_id, actor, detail)
                       VALUES ($1, 'onboarding_linked'::portal_audit_event, $2, $3, $4)"#,
                )
                .bind(Uuid::new_v4())
                .bind(portal_user_id)
                .bind(format!("lifecycle:{}", envelope.event_type))
                .bind(serde_json::json!({
                    "envelope_id": envelope.id,
                    "sapiens_user_id": sapiens_user,
                }))
                .execute(&self.pool)
                .await
                .map_err(|e| EventError::handler(HANDLER, format!("onboarding audit: {e}")))?;
            }
            None => {
                // No portal principal for this user — the audited
                // non-action (a new employee with no portal presence is
                // the NORMAL case; the audit row keeps it from being a
                // silent skip, per the digest growth-loop shape).
                sqlx::query(
                    r#"INSERT INTO portal.portal_audit_log
                         (id, event, actor, detail)
                       VALUES ($1, 'onboarding_no_invitation'::portal_audit_event, $2, $3)"#,
                )
                .bind(Uuid::new_v4())
                .bind(format!("lifecycle:{}", envelope.event_type))
                .bind(serde_json::json!({
                    "envelope_id": envelope.id,
                    "sapiens_user_id": sapiens_user,
                }))
                .execute(&self.pool)
                .await
                .map_err(|e| EventError::handler(HANDLER, format!("non-action audit: {e}")))?;
            }
        }
        Ok(())
    }

    /// The revocation arm (`sapiens.user.deactivated` / `.deleted` /
    /// `.anonymized`): find the principal by lifecycle link; revoke
    /// access + invalidate every live bearer (the rotation-to-invalid
    /// the W7-C6 sentence names). A sapiens user with no portal
    /// principal is a no-op (not an error, not audited — most sapiens
    /// users are employees with no portal presence; auditing every
    /// employee deletion would bury the signal in noise).
    async fn on_user_gone(&self, envelope: &IntegrationEventEnvelope) -> Result<(), EventError> {
        let payload: UserGonePayload = serde_json::from_value(envelope.payload.clone())
            .map_err(|e| {
                EventError::handler(
                    HANDLER,
                    format!("{} payload: {e} (envelope {})", envelope.event_type, envelope.id),
                )
            })?;
        let Ok(sapiens_user) = Uuid::parse_str(&payload.user_id) else {
            return Err(EventError::handler(
                HANDLER,
                format!("payload.user_id is not a uuid: {}", payload.user_id),
            ));
        };

        let principal: Option<(Uuid,)> = sqlx::query_as(
            r#"SELECT id FROM portal.portal_users
               WHERE sapiens_user_id = $1 AND (metadata->>'deleted_at') IS NULL"#,
        )
        .bind(sapiens_user)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| EventError::handler(HANDLER, format!("principal lookup: {e}")))?;
        let Some((user_id,)) = principal else {
            tracing::debug!(sapiens_user = %sapiens_user, "lifecycle revocation: no portal principal");
            return Ok(());
        };

        let flipped = self
            .access
            .revoke_access(user_id, &format!("lifecycle:{}", envelope.event_type), "lifecycle")
            .await
            .map_err(|e| EventError::handler(HANDLER, format!("revocation arm: {e}")))?;
        if !flipped {
            // A replayed envelope against an already-revoked principal:
            // performed edge already audited, nothing left to do.
            return Ok(());
        }

        sqlx::query(
            r#"INSERT INTO portal.portal_audit_log
                 (id, event, portal_user_id, actor, detail)
               VALUES ($1, 'lifecycle_access_revoked'::portal_audit_event, $2, $3, $4)"#,
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(format!("lifecycle:{}", envelope.event_type))
        .bind(serde_json::json!({
            "envelope_id": envelope.id,
            "sapiens_user_id": sapiens_user,
        }))
        .execute(&self.pool)
        .await
        .map_err(|e| EventError::handler(HANDLER, format!("revocation audit: {e}")))?;
        Ok(())
    }
}

#[async_trait]
impl IntegrationEventHandler for SapiensLifecycleHandler {
    async fn handle(&self, envelope: IntegrationEventEnvelope) -> Result<(), EventError> {
        match envelope.event_type.as_str() {
            "sapiens.user.created" => self.on_created(&envelope).await,
            "sapiens.user.deactivated" | "sapiens.user.deleted" | "sapiens.user.anonymized" => {
                self.on_user_gone(&envelope).await
            }
            other => Err(EventError::handler(
                HANDLER,
                format!("subscribed pattern matched an unhandled type: {other}"),
            )),
        }
    }

    fn event_patterns(&self) -> Vec<&'static str> {
        Self::PATTERNS.to_vec()
    }

    fn name(&self) -> &'static str {
        "PortalSapiensLifecycleHandler"
    }
}

//! One place this module records an audited fact.
//!
//! Consolidated from `portal.portal_audit_log` onto `auditlog.audit_trails`.
//!
//! Portal is the outlier among the modules that were folded in: its own table
//! keyed the subject with three separate nullable columns
//! (`portal_user_id`, `invite_id`, `token_id`) rather than a generic
//! `subject_type`/`subject_id` pair. The shared trail has one subject, so the
//! most specific id present becomes the subject and the rest fold into the diff
//! payload — nothing is dropped, and a per-record history finds the row under
//! the id a reader would look it up by.
//!
//! The actor here is free text (`"system"`, `"lifecycle:<event>"`), not a user
//! id, and it is passed explicitly: the trail's session-GUC default would
//! attribute a lifecycle handler's write to whoever's request happened to be
//! running, or to `system`, and both erase which handler acted.

use uuid::Uuid;

/// Record one audited fact in the caller's transaction.
pub async fn record_audit(
    exec: impl sqlx::Executor<'_, Database = sqlx::Postgres>,
    action: &str,
    actor: Option<&str>,
    portal_user_id: Option<Uuid>,
    invite_id: Option<Uuid>,
    token_id: Option<Uuid>,
    detail: serde_json::Value,
) -> Result<(), sqlx::Error> {
    // Most specific first: a row about a token is about that token, even when a
    // portal user is named alongside it.
    let (subject_type, subject_id) = match (token_id, invite_id, portal_user_id) {
        (Some(t), _, _) => (Some("portal.portal_tokens"), Some(t)),
        (_, Some(i), _) => (Some("portal.portal_invitations"), Some(i)),
        (_, _, Some(u)) => (Some("portal.portal_users"), Some(u)),
        _ => (None, None),
    };

    // Whatever did not become the subject still belongs on the row.
    let mut changed = match detail {
        serde_json::Value::Object(o) => o,
        other => {
            let mut m = serde_json::Map::new();
            m.insert("detail".into(), other);
            m
        }
    };
    for (key, id, used) in [
        ("portal_user_id", portal_user_id, subject_type == Some("portal.portal_users")),
        ("invite_id", invite_id, subject_type == Some("portal.portal_invitations")),
        ("token_id", token_id, subject_type == Some("portal.portal_tokens")),
    ] {
        if let (Some(v), false) = (id, used) {
            changed.insert(key.into(), serde_json::Value::String(v.to_string()));
        }
    }

    backbone_auditlog::application::service::append(
        exec,
        backbone_auditlog::application::service::AuditEvent {
            event_type: backbone_auditlog::domain::entity::AuditEventType::DataChange,
            action: action.to_string(),
            subject_type: subject_type.map(|s| s.to_string()),
            subject_id: subject_id.map(|id| id.to_string()),
            changed: Some(serde_json::Value::Object(changed)),
            reason: None,
            status: backbone_auditlog::domain::entity::AuditStatus::Success,
            actor: actor.map(|a| a.to_string()),
        },
    )
    .await
    .map(|_| ())
}

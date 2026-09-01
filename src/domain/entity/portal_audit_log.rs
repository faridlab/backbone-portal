use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::PortalAuditEvent;
use super::AuditMetadata;

/// Strongly-typed ID for PortalAuditLog
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PortalAuditLogId(pub Uuid);

impl PortalAuditLogId {
    pub fn new(id: Uuid) -> Self { Self(id) }
    pub fn generate() -> Self { Self(Uuid::new_v4()) }
    pub fn into_inner(self) -> Uuid { self.0 }
}

impl std::fmt::Display for PortalAuditLogId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for PortalAuditLogId {
    type Err = uuid::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl From<Uuid> for PortalAuditLogId {
    fn from(id: Uuid) -> Self { Self(id) }
}

impl From<PortalAuditLogId> for Uuid {
    fn from(id: PortalAuditLogId) -> Self { id.0 }
}

impl AsRef<Uuid> for PortalAuditLogId {
    fn as_ref(&self) -> &Uuid { &self.0 }
}

impl std::ops::Deref for PortalAuditLogId {
    type Target = Uuid;
    fn deref(&self) -> &Self::Target { &self.0 }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PortalAuditLog {
    pub id: Uuid,
    pub event: PortalAuditEvent,
    pub portal_user_id: Option<Uuid>,
    pub invite_id: Option<Uuid>,
    pub token_id: Option<Uuid>,
    pub actor: Option<String>,
    pub detail: Option<serde_json::Value>,
    pub occurred_at: DateTime<Utc>,
    #[serde(default)]
    #[sqlx(json)]
    pub metadata: AuditMetadata,
}

impl PortalAuditLog {
    /// Create a builder for PortalAuditLog
    pub fn builder() -> PortalAuditLogBuilder {
        <PortalAuditLogBuilder as Default>::default()
    }

    /// Create a new PortalAuditLog with required fields
    pub fn new(event: PortalAuditEvent, occurred_at: DateTime<Utc>) -> Self {
        Self {
            id: Uuid::new_v4(),
            event,
            portal_user_id: None,
            invite_id: None,
            token_id: None,
            actor: None,
            detail: None,
            occurred_at,
            metadata: AuditMetadata::default(),
        }
    }

    /// Get the entity's unique identifier
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Get a strongly-typed ID for this entity
    pub fn typed_id(&self) -> PortalAuditLogId {
        PortalAuditLogId(self.id)
    }

    /// Get when this entity was created
    pub fn created_at(&self) -> Option<&DateTime<Utc>> {
        self.metadata.created_at.as_ref()
    }

    /// Get when this entity was last updated
    pub fn updated_at(&self) -> Option<&DateTime<Utc>> {
        self.metadata.updated_at.as_ref()
    }

    /// Check if this entity is soft deleted
    pub fn is_deleted(&self) -> bool {
        self.metadata.deleted_at.is_some()
    }

    /// Check if this entity is active (not deleted)
    pub fn is_active(&self) -> bool {
        self.metadata.deleted_at.is_none()
    }

    /// Get when this entity was deleted
    pub fn deleted_at(&self) -> Option<&DateTime<Utc>> {
        self.metadata.deleted_at.as_ref()
    }

    /// Get who created this entity
    pub fn created_by(&self) -> Option<&Uuid> {
        self.metadata.created_by.as_ref()
    }

    /// Get who last updated this entity
    pub fn updated_by(&self) -> Option<&Uuid> {
        self.metadata.updated_by.as_ref()
    }

    /// Get who deleted this entity
    pub fn deleted_by(&self) -> Option<&Uuid> {
        self.metadata.deleted_by.as_ref()
    }


    // ==========================================================
    // Fluent Setters (with_* for optional fields)
    // ==========================================================

    /// Set the portal_user_id field (chainable)
    pub fn with_portal_user_id(mut self, value: Uuid) -> Self {
        self.portal_user_id = Some(value);
        self
    }

    /// Set the invite_id field (chainable)
    pub fn with_invite_id(mut self, value: Uuid) -> Self {
        self.invite_id = Some(value);
        self
    }

    /// Set the token_id field (chainable)
    pub fn with_token_id(mut self, value: Uuid) -> Self {
        self.token_id = Some(value);
        self
    }

    /// Set the actor field (chainable)
    pub fn with_actor(mut self, value: String) -> Self {
        self.actor = Some(value);
        self
    }

    /// Set the detail field (chainable)
    pub fn with_detail(mut self, value: serde_json::Value) -> Self {
        self.detail = Some(value);
        self
    }

    // ==========================================================
    // Partial Update
    // ==========================================================

    /// Apply partial updates from a map of field name to JSON value
    pub fn apply_patch(&mut self, fields: std::collections::HashMap<String, serde_json::Value>) {
        for (key, value) in fields {
            match key.as_str() {
                "event" => {
                    if let Ok(v) = serde_json::from_value(value) { self.event = v; }
                }
                "portal_user_id" => {
                    if let Ok(v) = serde_json::from_value(value) { self.portal_user_id = v; }
                }
                "invite_id" => {
                    if let Ok(v) = serde_json::from_value(value) { self.invite_id = v; }
                }
                "token_id" => {
                    if let Ok(v) = serde_json::from_value(value) { self.token_id = v; }
                }
                "actor" => {
                    if let Ok(v) = serde_json::from_value(value) { self.actor = v; }
                }
                "detail" => {
                    if let Ok(v) = serde_json::from_value(value) { self.detail = v; }
                }
                "occurred_at" => {
                    if let Ok(v) = serde_json::from_value(value) { self.occurred_at = v; }
                }
                _ => {} // ignore unknown fields
            }
        }
    }

    // <<< CUSTOM METHODS START >>>
    // <<< CUSTOM METHODS END >>>
}

impl super::Entity for PortalAuditLog {
    type Id = Uuid;

    fn entity_id(&self) -> &Self::Id {
        &self.id
    }

    fn entity_type() -> &'static str {
        "PortalAuditLog"
    }
}

impl backbone_core::PersistentEntity for PortalAuditLog {
    fn entity_id(&self) -> String {
        self.id.to_string()
    }
    fn set_entity_id(&mut self, id: String) {
        if let Ok(uuid) = uuid::Uuid::parse_str(&id) {
            self.id = uuid;
        }
    }
    fn created_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.metadata.created_at
    }
    fn set_created_at(&mut self, ts: chrono::DateTime<chrono::Utc>) {
        self.metadata.created_at = Some(ts);
    }
    fn updated_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.metadata.updated_at
    }
    fn set_updated_at(&mut self, ts: chrono::DateTime<chrono::Utc>) {
        self.metadata.updated_at = Some(ts);
    }
    fn deleted_at(&self) -> Option<chrono::DateTime<chrono::Utc>> {
        self.metadata.deleted_at
    }
    fn set_deleted_at(&mut self, ts: Option<chrono::DateTime<chrono::Utc>>) {
        self.metadata.deleted_at = ts;
    }
}

impl backbone_orm::EntityRepoMeta for PortalAuditLog {
    fn column_types() -> std::collections::HashMap<String, String> {
        let mut m = std::collections::HashMap::new();
        m.insert("id".to_string(), "uuid".to_string());
        m.insert("portal_user_id".to_string(), "uuid".to_string());
        m.insert("invite_id".to_string(), "uuid".to_string());
        m.insert("token_id".to_string(), "uuid".to_string());
        m.insert("event".to_string(), "portal_audit_event".to_string());
        m
    }
    fn search_fields() -> &'static [&'static str] {
        &[]
    }
}

/// Builder for PortalAuditLog entity
///
/// Provides a fluent API for constructing PortalAuditLog instances.
/// System fields (id, metadata, timestamps) are auto-initialized.
#[derive(Debug, Clone, Default)]
pub struct PortalAuditLogBuilder {
    event: Option<PortalAuditEvent>,
    portal_user_id: Option<Uuid>,
    invite_id: Option<Uuid>,
    token_id: Option<Uuid>,
    actor: Option<String>,
    detail: Option<serde_json::Value>,
    occurred_at: Option<DateTime<Utc>>,
}

impl PortalAuditLogBuilder {
    /// Set the event field (required)
    pub fn event(mut self, value: PortalAuditEvent) -> Self {
        self.event = Some(value);
        self
    }

    /// Set the portal_user_id field (optional)
    pub fn portal_user_id(mut self, value: Uuid) -> Self {
        self.portal_user_id = Some(value);
        self
    }

    /// Set the invite_id field (optional)
    pub fn invite_id(mut self, value: Uuid) -> Self {
        self.invite_id = Some(value);
        self
    }

    /// Set the token_id field (optional)
    pub fn token_id(mut self, value: Uuid) -> Self {
        self.token_id = Some(value);
        self
    }

    /// Set the actor field (optional)
    pub fn actor(mut self, value: String) -> Self {
        self.actor = Some(value);
        self
    }

    /// Set the detail field (optional)
    pub fn detail(mut self, value: serde_json::Value) -> Self {
        self.detail = Some(value);
        self
    }

    /// Set the occurred_at field (default: `Utc::now()`)
    pub fn occurred_at(mut self, value: DateTime<Utc>) -> Self {
        self.occurred_at = Some(value);
        self
    }

    /// Build the PortalAuditLog entity
    ///
    /// Returns Err if any required field without a default is missing.
    pub fn build(self) -> Result<PortalAuditLog, String> {
        let event = self.event.ok_or_else(|| "event is required".to_string())?;

        Ok(PortalAuditLog {
            id: Uuid::new_v4(),
            event,
            portal_user_id: self.portal_user_id,
            invite_id: self.invite_id,
            token_id: self.token_id,
            actor: self.actor,
            detail: self.detail,
            occurred_at: self.occurred_at.unwrap_or(Utc::now()),
            metadata: AuditMetadata::default(),
        })
    }
}

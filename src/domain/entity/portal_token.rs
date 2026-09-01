use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::PortalTokenStatus;
use super::AuditMetadata;

/// Strongly-typed ID for PortalToken
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PortalTokenId(pub Uuid);

impl PortalTokenId {
    pub fn new(id: Uuid) -> Self { Self(id) }
    pub fn generate() -> Self { Self(Uuid::new_v4()) }
    pub fn into_inner(self) -> Uuid { self.0 }
}

impl std::fmt::Display for PortalTokenId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for PortalTokenId {
    type Err = uuid::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl From<Uuid> for PortalTokenId {
    fn from(id: Uuid) -> Self { Self(id) }
}

impl From<PortalTokenId> for Uuid {
    fn from(id: PortalTokenId) -> Self { id.0 }
}

impl AsRef<Uuid> for PortalTokenId {
    fn as_ref(&self) -> &Uuid { &self.0 }
}

impl std::ops::Deref for PortalTokenId {
    type Target = Uuid;
    fn deref(&self) -> &Self::Target { &self.0 }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PortalToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_nonce: String,
    pub token_expires_at: DateTime<Utc>,
    pub status: PortalTokenStatus,
    pub rotated_at: Option<DateTime<Utc>>,
    pub rotated_to: Option<Uuid>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revocation_reason: Option<String>,
    pub last_used_at: Option<DateTime<Utc>>,
    #[serde(default)]
    #[sqlx(json)]
    pub metadata: AuditMetadata,
}

impl PortalToken {
    /// Create a builder for PortalToken
    pub fn builder() -> PortalTokenBuilder {
        <PortalTokenBuilder as Default>::default()
    }

    /// Create a new PortalToken with required fields
    pub fn new(user_id: Uuid, token_nonce: String, token_expires_at: DateTime<Utc>, status: PortalTokenStatus) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            token_nonce,
            token_expires_at,
            status,
            rotated_at: None,
            rotated_to: None,
            revoked_at: None,
            revocation_reason: None,
            last_used_at: None,
            metadata: AuditMetadata::default(),
        }
    }

    /// Get the entity's unique identifier
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Get a strongly-typed ID for this entity
    pub fn typed_id(&self) -> PortalTokenId {
        PortalTokenId(self.id)
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

    /// Get the current status
    pub fn status(&self) -> &PortalTokenStatus {
        &self.status
    }


    // ==========================================================
    // Fluent Setters (with_* for optional fields)
    // ==========================================================

    /// Set the rotated_at field (chainable)
    pub fn with_rotated_at(mut self, value: DateTime<Utc>) -> Self {
        self.rotated_at = Some(value);
        self
    }

    /// Set the rotated_to field (chainable)
    pub fn with_rotated_to(mut self, value: Uuid) -> Self {
        self.rotated_to = Some(value);
        self
    }

    /// Set the revoked_at field (chainable)
    pub fn with_revoked_at(mut self, value: DateTime<Utc>) -> Self {
        self.revoked_at = Some(value);
        self
    }

    /// Set the revocation_reason field (chainable)
    pub fn with_revocation_reason(mut self, value: String) -> Self {
        self.revocation_reason = Some(value);
        self
    }

    /// Set the last_used_at field (chainable)
    pub fn with_last_used_at(mut self, value: DateTime<Utc>) -> Self {
        self.last_used_at = Some(value);
        self
    }

    // ==========================================================
    // Partial Update
    // ==========================================================

    /// Apply partial updates from a map of field name to JSON value
    pub fn apply_patch(&mut self, fields: std::collections::HashMap<String, serde_json::Value>) {
        for (key, value) in fields {
            match key.as_str() {
                "user_id" => {
                    if let Ok(v) = serde_json::from_value(value) { self.user_id = v; }
                }
                "token_nonce" => {
                    if let Ok(v) = serde_json::from_value(value) { self.token_nonce = v; }
                }
                "token_expires_at" => {
                    if let Ok(v) = serde_json::from_value(value) { self.token_expires_at = v; }
                }
                "status" => {
                    if let Ok(v) = serde_json::from_value(value) { self.status = v; }
                }
                "rotated_at" => {
                    if let Ok(v) = serde_json::from_value(value) { self.rotated_at = v; }
                }
                "rotated_to" => {
                    if let Ok(v) = serde_json::from_value(value) { self.rotated_to = v; }
                }
                "revoked_at" => {
                    if let Ok(v) = serde_json::from_value(value) { self.revoked_at = v; }
                }
                "revocation_reason" => {
                    if let Ok(v) = serde_json::from_value(value) { self.revocation_reason = v; }
                }
                "last_used_at" => {
                    if let Ok(v) = serde_json::from_value(value) { self.last_used_at = v; }
                }
                _ => {} // ignore unknown fields
            }
        }
    }

    // <<< CUSTOM METHODS START >>>
    // <<< CUSTOM METHODS END >>>
}

impl super::Entity for PortalToken {
    type Id = Uuid;

    fn entity_id(&self) -> &Self::Id {
        &self.id
    }

    fn entity_type() -> &'static str {
        "PortalToken"
    }
}

impl backbone_core::PersistentEntity for PortalToken {
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

impl backbone_orm::EntityRepoMeta for PortalToken {
    fn column_types() -> std::collections::HashMap<String, String> {
        let mut m = std::collections::HashMap::new();
        m.insert("id".to_string(), "uuid".to_string());
        m.insert("user_id".to_string(), "uuid".to_string());
        m.insert("status".to_string(), "portal_token_status".to_string());
        m
    }
    fn search_fields() -> &'static [&'static str] {
        &["token_nonce"]
    }
    fn relations() -> &'static [(&'static str, &'static str, &'static str)] {
        &[("user", "portal_users", "userId")]
    }
}

/// Builder for PortalToken entity
///
/// Provides a fluent API for constructing PortalToken instances.
/// System fields (id, metadata, timestamps) are auto-initialized.
#[derive(Debug, Clone, Default)]
pub struct PortalTokenBuilder {
    user_id: Option<Uuid>,
    token_nonce: Option<String>,
    token_expires_at: Option<DateTime<Utc>>,
    status: Option<PortalTokenStatus>,
    rotated_at: Option<DateTime<Utc>>,
    rotated_to: Option<Uuid>,
    revoked_at: Option<DateTime<Utc>>,
    revocation_reason: Option<String>,
    last_used_at: Option<DateTime<Utc>>,
}

impl PortalTokenBuilder {
    /// Set the user_id field (required)
    pub fn user_id(mut self, value: Uuid) -> Self {
        self.user_id = Some(value);
        self
    }

    /// Set the token_nonce field (required)
    pub fn token_nonce(mut self, value: String) -> Self {
        self.token_nonce = Some(value);
        self
    }

    /// Set the token_expires_at field (required)
    pub fn token_expires_at(mut self, value: DateTime<Utc>) -> Self {
        self.token_expires_at = Some(value);
        self
    }

    /// Set the status field (default: `PortalTokenStatus::default()`)
    pub fn status(mut self, value: PortalTokenStatus) -> Self {
        self.status = Some(value);
        self
    }

    /// Set the rotated_at field (optional)
    pub fn rotated_at(mut self, value: DateTime<Utc>) -> Self {
        self.rotated_at = Some(value);
        self
    }

    /// Set the rotated_to field (optional)
    pub fn rotated_to(mut self, value: Uuid) -> Self {
        self.rotated_to = Some(value);
        self
    }

    /// Set the revoked_at field (optional)
    pub fn revoked_at(mut self, value: DateTime<Utc>) -> Self {
        self.revoked_at = Some(value);
        self
    }

    /// Set the revocation_reason field (optional)
    pub fn revocation_reason(mut self, value: String) -> Self {
        self.revocation_reason = Some(value);
        self
    }

    /// Set the last_used_at field (optional)
    pub fn last_used_at(mut self, value: DateTime<Utc>) -> Self {
        self.last_used_at = Some(value);
        self
    }

    /// Build the PortalToken entity
    ///
    /// Returns Err if any required field without a default is missing.
    pub fn build(self) -> Result<PortalToken, String> {
        let user_id = self.user_id.ok_or_else(|| "user_id is required".to_string())?;
        let token_nonce = self.token_nonce.ok_or_else(|| "token_nonce is required".to_string())?;
        let token_expires_at = self.token_expires_at.ok_or_else(|| "token_expires_at is required".to_string())?;

        Ok(PortalToken {
            id: Uuid::new_v4(),
            user_id,
            token_nonce,
            token_expires_at,
            status: self.status.unwrap_or_default(),
            rotated_at: self.rotated_at,
            rotated_to: self.rotated_to,
            revoked_at: self.revoked_at,
            revocation_reason: self.revocation_reason,
            last_used_at: self.last_used_at,
            metadata: AuditMetadata::default(),
        })
    }
}

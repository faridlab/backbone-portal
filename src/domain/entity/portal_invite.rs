use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::PortalInviteStatus;
use super::AuditMetadata;

/// Strongly-typed ID for PortalInvite
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PortalInviteId(pub Uuid);

impl PortalInviteId {
    pub fn new(id: Uuid) -> Self { Self(id) }
    pub fn generate() -> Self { Self(Uuid::new_v4()) }
    pub fn into_inner(self) -> Uuid { self.0 }
}

impl std::fmt::Display for PortalInviteId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for PortalInviteId {
    type Err = uuid::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl From<Uuid> for PortalInviteId {
    fn from(id: Uuid) -> Self { Self(id) }
}

impl From<PortalInviteId> for Uuid {
    fn from(id: PortalInviteId) -> Self { id.0 }
}

impl AsRef<Uuid> for PortalInviteId {
    fn as_ref(&self) -> &Uuid { &self.0 }
}

impl std::ops::Deref for PortalInviteId {
    type Target = Uuid;
    fn deref(&self) -> &Self::Target { &self.0 }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PortalInvite {
    pub id: Uuid,
    pub recipient_email: String,
    pub token_nonce: String,
    pub token_expires_at: DateTime<Utc>,
    pub status: PortalInviteStatus,
    pub granted_by: Option<Uuid>,
    pub redeemed_by: Option<Uuid>,
    pub redeemed_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revocation_reason: Option<String>,
    #[serde(default)]
    #[sqlx(json)]
    pub metadata: AuditMetadata,
}

impl PortalInvite {
    /// Create a builder for PortalInvite
    pub fn builder() -> PortalInviteBuilder {
        <PortalInviteBuilder as Default>::default()
    }

    /// Create a new PortalInvite with required fields
    pub fn new(recipient_email: String, token_nonce: String, token_expires_at: DateTime<Utc>, status: PortalInviteStatus) -> Self {
        Self {
            id: Uuid::new_v4(),
            recipient_email,
            token_nonce,
            token_expires_at,
            status,
            granted_by: None,
            redeemed_by: None,
            redeemed_at: None,
            revoked_at: None,
            revocation_reason: None,
            metadata: AuditMetadata::default(),
        }
    }

    /// Get the entity's unique identifier
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Get a strongly-typed ID for this entity
    pub fn typed_id(&self) -> PortalInviteId {
        PortalInviteId(self.id)
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
    pub fn status(&self) -> &PortalInviteStatus {
        &self.status
    }


    // ==========================================================
    // Fluent Setters (with_* for optional fields)
    // ==========================================================

    /// Set the granted_by field (chainable)
    pub fn with_granted_by(mut self, value: Uuid) -> Self {
        self.granted_by = Some(value);
        self
    }

    /// Set the redeemed_by field (chainable)
    pub fn with_redeemed_by(mut self, value: Uuid) -> Self {
        self.redeemed_by = Some(value);
        self
    }

    /// Set the redeemed_at field (chainable)
    pub fn with_redeemed_at(mut self, value: DateTime<Utc>) -> Self {
        self.redeemed_at = Some(value);
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

    // ==========================================================
    // Partial Update
    // ==========================================================

    /// Apply partial updates from a map of field name to JSON value
    pub fn apply_patch(&mut self, fields: std::collections::HashMap<String, serde_json::Value>) {
        for (key, value) in fields {
            match key.as_str() {
                "recipient_email" => {
                    if let Ok(v) = serde_json::from_value(value) { self.recipient_email = v; }
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
                "granted_by" => {
                    if let Ok(v) = serde_json::from_value(value) { self.granted_by = v; }
                }
                "redeemed_by" => {
                    if let Ok(v) = serde_json::from_value(value) { self.redeemed_by = v; }
                }
                "redeemed_at" => {
                    if let Ok(v) = serde_json::from_value(value) { self.redeemed_at = v; }
                }
                "revoked_at" => {
                    if let Ok(v) = serde_json::from_value(value) { self.revoked_at = v; }
                }
                "revocation_reason" => {
                    if let Ok(v) = serde_json::from_value(value) { self.revocation_reason = v; }
                }
                _ => {} // ignore unknown fields
            }
        }
    }

    // <<< CUSTOM METHODS START >>>
    // <<< CUSTOM METHODS END >>>
}

impl super::Entity for PortalInvite {
    type Id = Uuid;

    fn entity_id(&self) -> &Self::Id {
        &self.id
    }

    fn entity_type() -> &'static str {
        "PortalInvite"
    }
}

impl backbone_core::PersistentEntity for PortalInvite {
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

impl backbone_orm::EntityRepoMeta for PortalInvite {
    fn column_types() -> std::collections::HashMap<String, String> {
        let mut m = std::collections::HashMap::new();
        m.insert("id".to_string(), "uuid".to_string());
        m.insert("status".to_string(), "portal_invite_status".to_string());
        m
    }
    fn search_fields() -> &'static [&'static str] {
        &["recipient_email", "token_nonce"]
    }
}

/// Builder for PortalInvite entity
///
/// Provides a fluent API for constructing PortalInvite instances.
/// System fields (id, metadata, timestamps) are auto-initialized.
#[derive(Debug, Clone, Default)]
pub struct PortalInviteBuilder {
    recipient_email: Option<String>,
    token_nonce: Option<String>,
    token_expires_at: Option<DateTime<Utc>>,
    status: Option<PortalInviteStatus>,
    granted_by: Option<Uuid>,
    redeemed_by: Option<Uuid>,
    redeemed_at: Option<DateTime<Utc>>,
    revoked_at: Option<DateTime<Utc>>,
    revocation_reason: Option<String>,
}

impl PortalInviteBuilder {
    /// Set the recipient_email field (required)
    pub fn recipient_email(mut self, value: String) -> Self {
        self.recipient_email = Some(value);
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

    /// Set the status field (default: `PortalInviteStatus::default()`)
    pub fn status(mut self, value: PortalInviteStatus) -> Self {
        self.status = Some(value);
        self
    }

    /// Set the granted_by field (optional)
    pub fn granted_by(mut self, value: Uuid) -> Self {
        self.granted_by = Some(value);
        self
    }

    /// Set the redeemed_by field (optional)
    pub fn redeemed_by(mut self, value: Uuid) -> Self {
        self.redeemed_by = Some(value);
        self
    }

    /// Set the redeemed_at field (optional)
    pub fn redeemed_at(mut self, value: DateTime<Utc>) -> Self {
        self.redeemed_at = Some(value);
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

    /// Build the PortalInvite entity
    ///
    /// Returns Err if any required field without a default is missing.
    pub fn build(self) -> Result<PortalInvite, String> {
        let recipient_email = self.recipient_email.ok_or_else(|| "recipient_email is required".to_string())?;
        let token_nonce = self.token_nonce.ok_or_else(|| "token_nonce is required".to_string())?;
        let token_expires_at = self.token_expires_at.ok_or_else(|| "token_expires_at is required".to_string())?;

        Ok(PortalInvite {
            id: Uuid::new_v4(),
            recipient_email,
            token_nonce,
            token_expires_at,
            status: self.status.unwrap_or_default(),
            granted_by: self.granted_by,
            redeemed_by: self.redeemed_by,
            redeemed_at: self.redeemed_at,
            revoked_at: self.revoked_at,
            revocation_reason: self.revocation_reason,
            metadata: AuditMetadata::default(),
        })
    }
}

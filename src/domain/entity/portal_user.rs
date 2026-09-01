use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

use super::PortalUserStatus;
use super::AuditMetadata;

/// Strongly-typed ID for PortalUser
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PortalUserId(pub Uuid);

impl PortalUserId {
    pub fn new(id: Uuid) -> Self { Self(id) }
    pub fn generate() -> Self { Self(Uuid::new_v4()) }
    pub fn into_inner(self) -> Uuid { self.0 }
}

impl std::fmt::Display for PortalUserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for PortalUserId {
    type Err = uuid::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

impl From<Uuid> for PortalUserId {
    fn from(id: Uuid) -> Self { Self(id) }
}

impl From<PortalUserId> for Uuid {
    fn from(id: PortalUserId) -> Self { id.0 }
}

impl AsRef<Uuid> for PortalUserId {
    fn as_ref(&self) -> &Uuid { &self.0 }
}

impl std::ops::Deref for PortalUserId {
    type Target = Uuid;
    fn deref(&self) -> &Self::Target { &self.0 }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PortalUser {
    pub id: Uuid,
    pub email: String,
    pub display_name: Option<String>,
    pub status: PortalUserStatus,
    pub phone: Option<String>,
    pub street: Option<String>,
    pub street2: Option<String>,
    pub city: Option<String>,
    pub state_id: Option<String>,
    pub country_id: Option<String>,
    pub zip: Option<String>,
    pub vat: Option<String>,
    pub company_name: Option<String>,
    pub sapiens_user_id: Option<Uuid>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revocation_reason: Option<String>,
    pub archived_at: Option<DateTime<Utc>>,
    pub last_login_at: Option<DateTime<Utc>>,
    #[serde(default)]
    #[sqlx(json)]
    pub metadata: AuditMetadata,
}

impl PortalUser {
    /// Create a builder for PortalUser
    pub fn builder() -> PortalUserBuilder {
        <PortalUserBuilder as Default>::default()
    }

    /// Create a new PortalUser with required fields
    pub fn new(email: String, status: PortalUserStatus) -> Self {
        Self {
            id: Uuid::new_v4(),
            email,
            display_name: None,
            status,
            phone: None,
            street: None,
            street2: None,
            city: None,
            state_id: None,
            country_id: None,
            zip: None,
            vat: None,
            company_name: None,
            sapiens_user_id: None,
            revoked_at: None,
            revocation_reason: None,
            archived_at: None,
            last_login_at: None,
            metadata: AuditMetadata::default(),
        }
    }

    /// Get the entity's unique identifier
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Get a strongly-typed ID for this entity
    pub fn typed_id(&self) -> PortalUserId {
        PortalUserId(self.id)
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
    pub fn status(&self) -> &PortalUserStatus {
        &self.status
    }


    // ==========================================================
    // Fluent Setters (with_* for optional fields)
    // ==========================================================

    /// Set the display_name field (chainable)
    pub fn with_display_name(mut self, value: String) -> Self {
        self.display_name = Some(value);
        self
    }

    /// Set the phone field (chainable)
    pub fn with_phone(mut self, value: String) -> Self {
        self.phone = Some(value);
        self
    }

    /// Set the street field (chainable)
    pub fn with_street(mut self, value: String) -> Self {
        self.street = Some(value);
        self
    }

    /// Set the street2 field (chainable)
    pub fn with_street2(mut self, value: String) -> Self {
        self.street2 = Some(value);
        self
    }

    /// Set the city field (chainable)
    pub fn with_city(mut self, value: String) -> Self {
        self.city = Some(value);
        self
    }

    /// Set the state_id field (chainable)
    pub fn with_state_id(mut self, value: String) -> Self {
        self.state_id = Some(value);
        self
    }

    /// Set the country_id field (chainable)
    pub fn with_country_id(mut self, value: String) -> Self {
        self.country_id = Some(value);
        self
    }

    /// Set the zip field (chainable)
    pub fn with_zip(mut self, value: String) -> Self {
        self.zip = Some(value);
        self
    }

    /// Set the vat field (chainable)
    pub fn with_vat(mut self, value: String) -> Self {
        self.vat = Some(value);
        self
    }

    /// Set the company_name field (chainable)
    pub fn with_company_name(mut self, value: String) -> Self {
        self.company_name = Some(value);
        self
    }

    /// Set the sapiens_user_id field (chainable)
    pub fn with_sapiens_user_id(mut self, value: Uuid) -> Self {
        self.sapiens_user_id = Some(value);
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

    /// Set the archived_at field (chainable)
    pub fn with_archived_at(mut self, value: DateTime<Utc>) -> Self {
        self.archived_at = Some(value);
        self
    }

    /// Set the last_login_at field (chainable)
    pub fn with_last_login_at(mut self, value: DateTime<Utc>) -> Self {
        self.last_login_at = Some(value);
        self
    }

    // ==========================================================
    // Partial Update
    // ==========================================================

    /// Apply partial updates from a map of field name to JSON value
    pub fn apply_patch(&mut self, fields: std::collections::HashMap<String, serde_json::Value>) {
        for (key, value) in fields {
            match key.as_str() {
                "email" => {
                    if let Ok(v) = serde_json::from_value(value) { self.email = v; }
                }
                "display_name" => {
                    if let Ok(v) = serde_json::from_value(value) { self.display_name = v; }
                }
                "status" => {
                    if let Ok(v) = serde_json::from_value(value) { self.status = v; }
                }
                "phone" => {
                    if let Ok(v) = serde_json::from_value(value) { self.phone = v; }
                }
                "street" => {
                    if let Ok(v) = serde_json::from_value(value) { self.street = v; }
                }
                "street2" => {
                    if let Ok(v) = serde_json::from_value(value) { self.street2 = v; }
                }
                "city" => {
                    if let Ok(v) = serde_json::from_value(value) { self.city = v; }
                }
                "state_id" => {
                    if let Ok(v) = serde_json::from_value(value) { self.state_id = v; }
                }
                "country_id" => {
                    if let Ok(v) = serde_json::from_value(value) { self.country_id = v; }
                }
                "zip" => {
                    if let Ok(v) = serde_json::from_value(value) { self.zip = v; }
                }
                "vat" => {
                    if let Ok(v) = serde_json::from_value(value) { self.vat = v; }
                }
                "company_name" => {
                    if let Ok(v) = serde_json::from_value(value) { self.company_name = v; }
                }
                "sapiens_user_id" => {
                    if let Ok(v) = serde_json::from_value(value) { self.sapiens_user_id = v; }
                }
                "revoked_at" => {
                    if let Ok(v) = serde_json::from_value(value) { self.revoked_at = v; }
                }
                "revocation_reason" => {
                    if let Ok(v) = serde_json::from_value(value) { self.revocation_reason = v; }
                }
                "archived_at" => {
                    if let Ok(v) = serde_json::from_value(value) { self.archived_at = v; }
                }
                "last_login_at" => {
                    if let Ok(v) = serde_json::from_value(value) { self.last_login_at = v; }
                }
                _ => {} // ignore unknown fields
            }
        }
    }

    // <<< CUSTOM METHODS START >>>
    // <<< CUSTOM METHODS END >>>
}

impl super::Entity for PortalUser {
    type Id = Uuid;

    fn entity_id(&self) -> &Self::Id {
        &self.id
    }

    fn entity_type() -> &'static str {
        "PortalUser"
    }
}

impl backbone_core::PersistentEntity for PortalUser {
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

impl backbone_orm::EntityRepoMeta for PortalUser {
    fn column_types() -> std::collections::HashMap<String, String> {
        let mut m = std::collections::HashMap::new();
        m.insert("id".to_string(), "uuid".to_string());
        m.insert("sapiens_user_id".to_string(), "uuid".to_string());
        m.insert("status".to_string(), "portal_user_status".to_string());
        m
    }
    fn search_fields() -> &'static [&'static str] {
        &["email"]
    }
}

/// Builder for PortalUser entity
///
/// Provides a fluent API for constructing PortalUser instances.
/// System fields (id, metadata, timestamps) are auto-initialized.
#[derive(Debug, Clone, Default)]
pub struct PortalUserBuilder {
    email: Option<String>,
    display_name: Option<String>,
    status: Option<PortalUserStatus>,
    phone: Option<String>,
    street: Option<String>,
    street2: Option<String>,
    city: Option<String>,
    state_id: Option<String>,
    country_id: Option<String>,
    zip: Option<String>,
    vat: Option<String>,
    company_name: Option<String>,
    sapiens_user_id: Option<Uuid>,
    revoked_at: Option<DateTime<Utc>>,
    revocation_reason: Option<String>,
    archived_at: Option<DateTime<Utc>>,
    last_login_at: Option<DateTime<Utc>>,
}

impl PortalUserBuilder {
    /// Set the email field (required)
    pub fn email(mut self, value: String) -> Self {
        self.email = Some(value);
        self
    }

    /// Set the display_name field (optional)
    pub fn display_name(mut self, value: String) -> Self {
        self.display_name = Some(value);
        self
    }

    /// Set the status field (default: `PortalUserStatus::default()`)
    pub fn status(mut self, value: PortalUserStatus) -> Self {
        self.status = Some(value);
        self
    }

    /// Set the phone field (optional)
    pub fn phone(mut self, value: String) -> Self {
        self.phone = Some(value);
        self
    }

    /// Set the street field (optional)
    pub fn street(mut self, value: String) -> Self {
        self.street = Some(value);
        self
    }

    /// Set the street2 field (optional)
    pub fn street2(mut self, value: String) -> Self {
        self.street2 = Some(value);
        self
    }

    /// Set the city field (optional)
    pub fn city(mut self, value: String) -> Self {
        self.city = Some(value);
        self
    }

    /// Set the state_id field (optional)
    pub fn state_id(mut self, value: String) -> Self {
        self.state_id = Some(value);
        self
    }

    /// Set the country_id field (optional)
    pub fn country_id(mut self, value: String) -> Self {
        self.country_id = Some(value);
        self
    }

    /// Set the zip field (optional)
    pub fn zip(mut self, value: String) -> Self {
        self.zip = Some(value);
        self
    }

    /// Set the vat field (optional)
    pub fn vat(mut self, value: String) -> Self {
        self.vat = Some(value);
        self
    }

    /// Set the company_name field (optional)
    pub fn company_name(mut self, value: String) -> Self {
        self.company_name = Some(value);
        self
    }

    /// Set the sapiens_user_id field (optional)
    pub fn sapiens_user_id(mut self, value: Uuid) -> Self {
        self.sapiens_user_id = Some(value);
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

    /// Set the archived_at field (optional)
    pub fn archived_at(mut self, value: DateTime<Utc>) -> Self {
        self.archived_at = Some(value);
        self
    }

    /// Set the last_login_at field (optional)
    pub fn last_login_at(mut self, value: DateTime<Utc>) -> Self {
        self.last_login_at = Some(value);
        self
    }

    /// Build the PortalUser entity
    ///
    /// Returns Err if any required field without a default is missing.
    pub fn build(self) -> Result<PortalUser, String> {
        let email = self.email.ok_or_else(|| "email is required".to_string())?;

        Ok(PortalUser {
            id: Uuid::new_v4(),
            email,
            display_name: self.display_name,
            status: self.status.unwrap_or_default(),
            phone: self.phone,
            street: self.street,
            street2: self.street2,
            city: self.city,
            state_id: self.state_id,
            country_id: self.country_id,
            zip: self.zip,
            vat: self.vat,
            company_name: self.company_name,
            sapiens_user_id: self.sapiens_user_id,
            revoked_at: self.revoked_at,
            revocation_reason: self.revocation_reason,
            archived_at: self.archived_at,
            last_login_at: self.last_login_at,
            metadata: AuditMetadata::default(),
        })
    }
}

//! `PortalDocumentSurface` — the exported public trait surface, the
//! W7-C4 artifact (hand-written; user-owned; see `metaphor.codegen.yaml`).
//!
//! This is the named contract the website family (WB-3) and the
//! download center (WB-4) consume — a TRAIT, not route handlers: the
//! module exports verbs + read models; the composing host decides how
//! they surface (its own HTTP shapes, queued jobs, whatever).
//!
//! ## Declared read models — ownership-scoped, never sudo-browse
//!
//! Every read is keyed by `principal.user_id` (the verified bearer's
//! own id) in the SQL `WHERE` clause itself — there is NO verb on this
//! surface that browses another principal's rows, no officer-browse
//! escape hatch, and no caller-supplied id parameter anywhere. The
//! isolation is structural: a caller cannot ask for someone else's
//! document because the ask has no place to put it.
//!
//! ## Per-verb field whitelists (the PI-04 shape)
//!
//! The legacy portal-core carried a 12-field customer allowlist: name,
//! phone, email, street, street2, city, state_id, country_id, zip,
//! zipcode, vat, company_name. This port applies two recorded
//! decisions:
//!
//! 1. **zip/zipcode collapse** — the legacy shape carried BOTH spellings
//!    (an upstream family artifact); the port collapses them to the one
//!    `zip` column, 12 -> 11 fields.
//! 2. **email is not self-service writable** — every capability in this
//!    module BINDS to the email (the invitation grant, the recipient
//!    HMAC, the login identity, the unique-on-live index), so a
//!    self-service email change would silently orphan the credential
//!    chain. Email appears in the READ view and moves only through
//!    officer verbs (new invitation + revocation), 11 -> 10 writable.
//!
//! [`WRITABLE_DETAIL_FIELDS`] is the single declared list;
//! [`PortalDetailPatch`] is its typed form — a field the patch struct
//! does not carry is a field no verb can write, and an unknown key in
//! the HTTP body is dropped at the route, never forwarded here.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::application::service::portal_error::PortalError;
use crate::application::service::token_service::PortalPrincipal;

/// The declared self-service writable whitelist — PI-04's 12-field
/// shape after the two recorded decisions (see the module doc). This
/// constant IS the contract; the patch struct and the update verb are
/// its typed projection.
pub const WRITABLE_DETAIL_FIELDS: [&str; 10] = [
    "display_name", // PI-04 "name"
    "phone",
    "street",
    "street2",
    "city",
    "state_id",
    "country_id",
    "zip", // PI-04 "zip" + "zipcode" collapsed
    "vat",
    "company_name",
];

/// The declared read view over the principal's own record (the
/// `my_details` read model). Serializes directly — it is the wire shape
/// of `GET /me`.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct PortalAccountView {
    pub user_id: Uuid,
    pub email: String,
    pub display_name: Option<String>,
    pub status: String,
    pub phone: Option<String>,
    pub street: Option<String>,
    pub street2: Option<String>,
    pub city: Option<String>,
    pub state_id: Option<String>,
    pub country_id: Option<String>,
    pub zip: Option<String>,
    pub vat: Option<String>,
    pub company_name: Option<String>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// The typed self-service patch — exactly the writable whitelist, each
/// arm `None` = leave unchanged. A field absent from this struct cannot
/// be written through any verb on this surface. Deserialization ignores
/// unknown keys by construction (serde's default): an off-whitelist key
/// in the request body simply never maps to an arm.
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PortalDetailPatch {
    pub display_name: Option<String>,
    pub phone: Option<String>,
    pub street: Option<String>,
    pub street2: Option<String>,
    pub city: Option<String>,
    pub state_id: Option<String>,
    pub country_id: Option<String>,
    pub zip: Option<String>,
    pub vat: Option<String>,
    pub company_name: Option<String>,
}

impl PortalDetailPatch {
    /// Is every arm `None` (a no-op the update verb refuses)?
    pub fn is_empty(&self) -> bool {
        self.display_name.is_none()
            && self.phone.is_none()
            && self.street.is_none()
            && self.street2.is_none()
            && self.city.is_none()
            && self.state_id.is_none()
            && self.country_id.is_none()
            && self.zip.is_none()
            && self.vat.is_none()
            && self.company_name.is_none()
    }

    /// Validate the set arms against the schema's declared `@max`
    /// lengths (the same numbers the migration carries). Returns the
    /// first violation; callers refuse the whole patch (never a partial
    /// write) on any violation.
    pub fn validate(&self) -> Result<(), PortalError> {
        let checks: [(&str, Option<&String>, usize); 10] = [
            ("display_name", self.display_name.as_ref(), 120),
            ("phone", self.phone.as_ref(), 40),
            ("street", self.street.as_ref(), 160),
            ("street2", self.street2.as_ref(), 160),
            ("city", self.city.as_ref(), 80),
            ("state_id", self.state_id.as_ref(), 80),
            ("country_id", self.country_id.as_ref(), 8),
            ("zip", self.zip.as_ref(), 24),
            ("vat", self.vat.as_ref(), 40),
            ("company_name", self.company_name.as_ref(), 120),
        ];
        for (name, value, max) in checks {
            if let Some(v) = value {
                if v.len() > max {
                    return Err(PortalError::InvalidInput(format!(
                        "{name} exceeds the maximum length of {max}"
                    )));
                }
            }
        }
        Ok(())
    }
}

/// One entry of the `my_access_history` read model — the principal's
/// own credential-lifecycle audit trail.
#[derive(Debug, Clone)]
pub struct PortalAccessEvent {
    pub event: String,
    pub occurred_at: DateTime<Utc>,
    pub actor: Option<String>,
}

/// The events the access-history read model exposes (the principal's
/// own security timeline — nothing about other principals, nothing
/// about officer actions the principal cannot see the subject of).
/// Documentation anchor for the verb's SQL — kept in sync by the
/// surface probes.
#[allow(dead_code)]
const ACCESS_HISTORY_EVENTS: [&str; 5] = [
    "login_succeeded",
    "login_refused",
    "bearer_minted",
    "bearer_rotated",
    "bearer_revoked",
];

/// The exported document surface — THE contract WB-3/WB-4 consume.
/// Implementations MUST scope every statement to `principal.user_id`
/// (see [`PgPortalSurface`] for the reference SQL shape); the
/// isolation probe class asserts it.
#[async_trait]
pub trait PortalDocumentSurface: Send + Sync {
    /// The declared writable whitelist (PI-04 shape; see the module
    /// doc). Exposed so consumers and probes can assert the contract,
    /// not restate it.
    fn writable_detail_fields(&self) -> &'static [&'static str] {
        &WRITABLE_DETAIL_FIELDS
    }

    /// `my_details` — the principal's own record. Ownership-scoped:
    /// reads `WHERE id = principal.user_id` and only a live, active
    /// row.
    async fn my_details(&self, principal: &PortalPrincipal)
        -> Result<PortalAccountView, PortalError>;

    /// `update_my_details` — the single self-service write verb, over
    /// the declared whitelist only. Refuses an empty patch. The write
    /// stamps `metadata.updated_by` with the principal's own id (the
    /// PISO-2 identity-stamp property) and returns the fresh view.
    async fn update_my_details(
        &self,
        principal: &PortalPrincipal,
        patch: PortalDetailPatch,
    ) -> Result<PortalAccountView, PortalError>;

    /// `my_access_history` — the principal's own credential-lifecycle
    /// audit trail (logins, mints, rotations, revocations), newest
    /// first, capped by `limit` (<= 100).
    async fn my_access_history(
        &self,
        principal: &PortalPrincipal,
        limit: i64,
    ) -> Result<Vec<PortalAccessEvent>, PortalError>;
}

/// The Postgres reference implementation of the surface.
pub struct PgPortalSurface {
    pool: PgPool,
}

impl PgPortalSurface {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn view(
        &self,
        user_id: Uuid,
    ) -> Result<PortalAccountView, PortalError> {
        let v = sqlx::query_as::<_, (
            Uuid, String, Option<String>, String,
            Option<String>, Option<String>, Option<String>, Option<String>,
            Option<String>, Option<String>, Option<String>, Option<String>,
            Option<String>, Option<DateTime<Utc>>, Option<String>,
        )>(
            r#"SELECT id, email, display_name, status::text,
                      phone, street, street2, city,
                      state_id, country_id, zip, vat, company_name,
                      last_login_at,
                      (metadata->>'updated_at')
               FROM portal.portal_users
               WHERE id = $1 AND status = 'active'
                 AND (metadata->>'deleted_at') IS NULL"#,
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| PortalError::NotFound(format!("principal {user_id}")))?;
        let updated_at = v
            .14
            .as_deref()
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|d| d.with_timezone(&Utc));
        Ok(PortalAccountView {
            user_id: v.0,
            email: v.1,
            display_name: v.2,
            status: v.3,
            phone: v.4,
            street: v.5,
            street2: v.6,
            city: v.7,
            state_id: v.8,
            country_id: v.9,
            zip: v.10,
            vat: v.11,
            company_name: v.12,
            last_login_at: v.13,
            updated_at,
        })
    }
}

#[async_trait]
impl PortalDocumentSurface for PgPortalSurface {
    async fn my_details(
        &self,
        principal: &PortalPrincipal,
    ) -> Result<PortalAccountView, PortalError> {
        self.view(principal.user_id).await
    }

    async fn update_my_details(
        &self,
        principal: &PortalPrincipal,
        patch: PortalDetailPatch,
    ) -> Result<PortalAccountView, PortalError> {
        if patch.is_empty() {
            return Err(PortalError::InvalidInput(
                "the patch sets no field".into(),
            ));
        }
        patch.validate()?;

        // Whitelist-only SET construction: the column list is compiled
        // from the patch struct's OWN arms — an arbitrary column name
        // cannot reach this query.
        let mut builder = sqlx::QueryBuilder::new("UPDATE portal.portal_users SET ");
        let mut first = true;
        macro_rules! set_arm {
            ($col:literal, $value:expr) => {
                if let Some(v) = $value {
                    if !first {
                        builder.push(", ");
                    }
                    builder.push($col);
                    builder.push(" = ");
                    builder.push_bind(v);
                    first = false;
                }
            };
        }
        set_arm!("display_name", patch.display_name.clone());
        set_arm!("phone", patch.phone.clone());
        set_arm!("street", patch.street.clone());
        set_arm!("street2", patch.street2.clone());
        set_arm!("city", patch.city.clone());
        set_arm!("state_id", patch.state_id.clone());
        set_arm!("country_id", patch.country_id.clone());
        set_arm!("zip", patch.zip.clone());
        set_arm!("vat", patch.vat.clone());
        set_arm!("company_name", patch.company_name.clone());
        // The PISO-2 identity stamp: the write records WHO wrote it —
        // the acting principal, always.
        builder.push(", metadata = jsonb_set(metadata, '{updated_by}', to_jsonb(");
        builder.push_bind(principal.user_id.to_string());
        builder.push("::text)) WHERE id = ");
        builder.push_bind(principal.user_id);
        builder.push(" AND status = 'active' AND (metadata->>'deleted_at') IS NULL");

        let rows = builder.build().execute(&self.pool).await?.rows_affected();
        if rows != 1 {
            // Not found (or not active) THROUGH THE PRINCIPAL'S OWN KEY —
            // the ownership-scoped refusal; no id ever crossed the seam.
            return Err(PortalError::NotFound(format!(
                "principal {}",
                principal.user_id
            )));
        }
        self.view(principal.user_id).await
    }

    async fn my_access_history(
        &self,
        principal: &PortalPrincipal,
        limit: i64,
    ) -> Result<Vec<PortalAccessEvent>, PortalError> {
        let limit = limit.clamp(1, 100);
        let rows = sqlx::query_as::<_, (String, DateTime<Utc>, Option<String>)>(
            r#"SELECT event::text, occurred_at, actor
               FROM portal.portal_audit_log
               WHERE portal_user_id = $1
                 AND event IN ('login_succeeded', 'login_refused',
                               'bearer_minted', 'bearer_rotated',
                               'bearer_revoked')
               ORDER BY occurred_at DESC
               LIMIT $2"#,
        )
        .bind(principal.user_id)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows
            .into_iter()
            .map(|(event, occurred_at, actor)| PortalAccessEvent { event, occurred_at, actor })
            .collect())
    }
}

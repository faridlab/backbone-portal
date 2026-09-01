//! The portal module's typed error surface (hand-written; user-owned; see
//! `metaphor.codegen.yaml`).
//!
//! Two rules shape every variant:
//!
//! 1. **No oracle.** The auth family's refusals share ONE body — unknown
//!    email, wrong credential, revoked principal, and unconfigured
//!    verifier-indistinguishable outcomes all map to
//!    [`PortalError::InvalidCredentials`] at the route (the PI-23
//!    enumeration fix; the login row in the audit log records the FACT,
//!    never which half failed). The typed variants below exist for the
//!    service layer and probes; the route mapping collapses them.
//! 2. **No silent degradation.** A missing secret or an unwired port is a
//!    LOUD typed failure (the engagement/survey secret posture and the
//!    survey certification-port posture) — never a fallback that quietly
//!    weakens a credential.

use thiserror::Error;

/// The module error enum.
#[derive(Debug, Error)]
pub enum PortalError {
    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),

    #[error("internal error: {0}")]
    Internal(String),

    /// The bearer secret (`PORTAL_BEARER_TOKEN_SECRET`) is not configured.
    /// Minting/rotating refuses loudly — a silent empty key would let
    /// anyone forge bearers.
    #[error("portal bearer token secret is not configured")]
    BearerSecretNotConfigured,

    /// The recipient-HMAC secret (`PORTAL_RECIPIENT_TOKEN_SECRET`) is not
    /// configured.
    #[error("portal recipient token secret is not configured")]
    RecipientSecretNotConfigured,

    /// The shared refusal for EVERY credential rejection: unknown token,
    /// forged MAC, expired, rotated, revoked, revoked principal, malformed.
    /// One shape — no oracle (PI-21/PI-23).
    #[error("credential refused")]
    CredentialRefused,

    /// The uniform login/signup credential refusal (unknown email and
    /// wrong password are indistinguishable here BY CONTRACT — the
    /// credential port must preserve that).
    #[error("invalid credentials")]
    InvalidCredentials,

    /// The login throttle fired (Tier B, per identity AND per IP).
    #[error("too many attempts; retry after {retry_after_seconds}s")]
    RateLimited {
        retry_after_seconds: i64,
    },

    /// The signup kill-switch is closed (the DEFAULT posture; PI-09/PI-18).
    #[error("signup is not open on this service")]
    SignupClosed,

    /// The credential verifier port is unwired — the host composed no
    /// verifier, so password-shaped logins refuse loudly rather than
    /// pretending (the fail-closed port class).
    #[error("credential verifier not composed (host must register one)")]
    CredentialPortNotComposed,

    /// Input validation refusal (typed, surfaced — not a shared-body case;
    /// the caller sent a malformed request).
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// A requested record does not exist (officer verbs).
    #[error("not found: {0}")]
    NotFound(String),
}

impl PortalError {
    /// The HTTP status the route layer maps this error to.
    pub fn http_status(&self) -> u16 {
        match self {
            PortalError::Db(_) | PortalError::Internal(_) => 500,
            PortalError::BearerSecretNotConfigured
            | PortalError::RecipientSecretNotConfigured => 500,
            PortalError::CredentialRefused | PortalError::InvalidCredentials => 401,
            PortalError::RateLimited { .. } => 429,
            PortalError::SignupClosed => 403,
            PortalError::CredentialPortNotComposed => 503,
            PortalError::InvalidInput(_) => 400,
            PortalError::NotFound(_) => 404,
        }
    }

    /// The stable machine code the route layer emits.
    pub fn code(&self) -> &'static str {
        match self {
            PortalError::Db(_) => "portal_internal_error",
            PortalError::Internal(_) => "portal_internal_error",
            PortalError::BearerSecretNotConfigured => "portal_internal_error",
            PortalError::RecipientSecretNotConfigured => "portal_internal_error",
            PortalError::CredentialRefused => "portal_credential_refused",
            PortalError::InvalidCredentials => "portal_invalid_credentials",
            PortalError::RateLimited { .. } => "portal_rate_limited",
            PortalError::SignupClosed => "portal_signup_closed",
            PortalError::CredentialPortNotComposed => "portal_credential_port_not_composed",
            PortalError::InvalidInput(_) => "portal_invalid_input",
            PortalError::NotFound(_) => "portal_not_found",
        }
    }
}

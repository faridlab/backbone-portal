//! The fail-closed credential seam (hand-written; user-owned; see
//! `metaphor.codegen.yaml`).
//!
//! The portal owns WHO its principals are (portal-owned identity home —
//! the W7 identity-home ruling) but sapiens owns the password hashes.
//! Password-shaped login therefore crosses a PORT, not an edge read:
//! the module owns the trait + the refusing default; the composing host
//! registers the verifier (an argon2-against-sapiens adapter, or
//! whatever future session surface lands). Unwired = the loud typed
//! [`crate::application::service::portal_error::PortalError::CredentialPortNotComposed`]
//! refusal + an audited `credential_port_not_composed` row — the survey
//! certification-port class, never a silent weaken-to-allow.
//!
//! ## The de-oracle contract (binding on every implementation)
//!
//! [`PortalCredentialVerifier::verify`] answers `Ok(true)` /
//! `Ok(false)` / `Err(internal)`. The implementation MUST make
//! **unknown email** and **wrong password** indistinguishable — both
//! `Ok(false)`, ideally at equal cost (a plain "unknown email" early
//! return leaks an existence oracle by timing; the host seat's rider on
//! the live session bridge covers the known instance of that defect).

use std::sync::Mutex;

use async_trait::async_trait;

/// The check request: the email (normalized by the caller) and the
/// presented secret (a password-shaped credential — the exact form is
/// the verifier's business; the module never interprets it).
#[derive(Debug, Clone)]
pub struct CredentialCheck<'a> {
    pub email: &'a str,
    pub presented: &'a str,
}

/// An internal verifier failure (infrastructure trouble — NOT a
/// wrong-password verdict). Distinct from `Ok(false)` by design so the
/// route can answer 500 rather than minting a fake 401.
#[derive(Debug, thiserror::Error)]
#[error("credential verifier internal failure: {0}")]
pub struct CredentialVerifierError(pub String);

/// The host-installed verifier.
#[async_trait]
pub trait PortalCredentialVerifier: Send + Sync {
    /// `Ok(true)` = the credential verifies for this email;
    /// `Ok(false)` = it does not (unknown email and wrong password are
    /// indistinguishable HERE — see the module doc contract);
    /// `Err` = internal failure.
    async fn verify(&self, check: CredentialCheck<'_>) -> Result<bool, CredentialVerifierError>;
}

/// The installable slot (deny-by-default until the host calls
/// `install`). `Clone`-able so services and the module can share one.
#[derive(Clone, Default)]
pub struct CredentialVerifierSlot {
    inner: std::sync::Arc<Mutex<Option<std::sync::Arc<dyn PortalCredentialVerifier>>>>,
}

impl CredentialVerifierSlot {
    pub fn new() -> Self {
        Self::default()
    }

    /// Install the host's verifier (compose time, once).
    pub fn install(&self, verifier: std::sync::Arc<dyn PortalCredentialVerifier>) {
        *self
            .inner
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = Some(verifier);
    }

    /// Is a verifier installed?
    pub fn is_wired(&self) -> bool {
        self.inner
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .is_some()
    }

    /// Run a check through the installed verifier; `None` = not
    /// composed (the caller maps that to the loud typed refusal).
    pub async fn check(
        &self,
        check: CredentialCheck<'_>,
    ) -> Option<Result<bool, CredentialVerifierError>> {
        let verifier = self
            .inner
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        match verifier {
            Some(v) => Some(v.verify(check).await),
            None => None,
        }
    }
}

impl std::fmt::Debug for CredentialVerifierSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CredentialVerifierSlot")
            .field("wired", &self.is_wired())
            .finish()
    }
}

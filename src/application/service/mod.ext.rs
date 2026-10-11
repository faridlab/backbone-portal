// This directory's extension (hand-written; ADR-0031). The generated mod.rs (lib.rs for
// src/) pulls it in with include!, so its lines resolve against this directory.

// The shared-audit-trail funnel. Inside the markers so a regeneration
// cannot drop the declaration and leave the file orphaned.
pub mod audit;
// Hand-written portal services (user-owned files — see
// metaphor.codegen.yaml; NOT GenericCrudService aliases).
pub mod portal_error;
pub mod token_service;
pub mod invite_service;
pub mod policy_service;
pub mod credential_port;
pub mod access_service;
pub mod lifecycle_handler;
pub mod portal_surface;
pub use access_service::AccessService;
pub use credential_port::{
    CredentialCheck, CredentialVerifierError, CredentialVerifierSlot, PortalCredentialVerifier,
};
pub use invite_service::InviteService;
pub use lifecycle_handler::SapiensLifecycleHandler;
pub use policy_service::PolicyService;
pub use portal_error::PortalError;
pub use portal_surface::{
    PgPortalSurface, PortalAccessEvent, PortalAccountView, PortalDetailPatch,
    PortalDocumentSurface, WRITABLE_DETAIL_FIELDS,
};
pub use token_service::{
    AttemptBook, BearerRow, ParsedCapability, PortalPrincipal, RecipientCredential, TokenService,
    PORTAL_BEARER_SECRET_ENV, PORTAL_RECIPIENT_SECRET_ENV,
};

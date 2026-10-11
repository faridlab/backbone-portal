// This directory's extension (hand-written; ADR-0031). The generated mod.rs (lib.rs for
// src/) pulls it in with include!, so its lines resolve against this directory.

// The hand-written public auth tree (user-owned — see
// metaphor.codegen.yaml). The module does NOT self-mount it; the host
// nests the exported router at /api/v1/portal behind its gates.
pub mod public_routes;
pub use public_routes::{portal_public_routes, PortalPublicState};

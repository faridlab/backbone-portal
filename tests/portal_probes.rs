//! The portal domain module's behavioral gate.
//!
//! Five probe classes, each against its own disposable scratch
//! Postgres (port 5433 — never a live service database):
//!
//! - `piso_isolation` — the MANDATORY customer-isolation class
//!   (ownership-scoped reads, session-identity write stamps, the
//!   bearer-as-identity arm), ported from the predecessor's
//!   `tests/portal_isolation.rs` (preserved at tag v0.1.1).
//! - `credentials` — the two INDEPENDENT Tier A credentials: the
//!   expiring rotatable bearer and the stateless per-recipient HMAC.
//! - `invites` — the invitation capability: one-shot redemption,
//!   recipient binding, the explicit revocation list, expiry.
//! - `policy` — the kill-switchable signup policy (default OFF,
//!   fail-closed), the fail-closed credential port, the Tier B
//!   throttle at the access verbs.
//! - `lifecycle` — the sapiens lifecycle subscription (onboarding
//!   link + deactivate/delete revocation) against the published
//!   v0.2.4 event contract, exercised with fixture envelopes.
//!
//! Fail-hard contract: see `probes/common/mod.rs` — a skipped probe
//! is a failed probe.

mod probes;

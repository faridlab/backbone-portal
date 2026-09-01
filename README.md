# backbone-portal

The portal identity domain module: portal-owned principal records,
two independent Tier A credentials, an invitation-only signup policy,
and the declared read-model surface downstream website modules
consume. Schema name: `portal` (routes mount under `/api/v1/portal`).

## What this module is — and is not

- **Is**: a domain module in the backbone family. Identity TABLES for
  portal principals live here (`portal.portal_users`,
  `portal.portal_tokens`, `portal.portal_invites`,
  `portal.portal_signup_policies`, `portal.portal_audit_log`); the
  ACCOUNT lifecycle lives in
  `backbone-sapiens`, and this module subscribes to its integration
  events (created → onboarding link; deactivated/deleted → access
  revocation).
- **Is not**: a consuming service. There is no service skeleton, no
  sales/billing/support wiring, and no self-mounted router. The module
  exports a gated router the HOST composes; it never listens on its
  own and never registers anything under `/api/v1/auth` (the host's
  auth bridge lives there).
- **Dependencies**: `backbone-framework` (core/orm/auth/messaging) and
  `backbone-rate-limit`, all pinned at tag `v2.7.11`, plus exactly one
  module edge: `backbone-sapiens` at tag `v0.2.4` (the event contract
  the lifecycle handler codes against). No edges to sales-graph or
  website-family modules — websites consume this module's TRAIT
  surface, they are not its dependencies.

## The two-credential contract (Tier A)

Every principal holds TWO credentials that rotate INDEPENDENTLY —
rotating either never invalidates the other (the property
`tests/probes/credentials.rs::credentials_rotate_independently`
proves in both directions):

1. **Bearer capability** `{id}.{nonce}.{exp}.{mac}` — HMAC-SHA256 over
   `(id, nonce, grant, exp)` under `PORTAL_BEARER_TOKEN_SECRET`.
   Stored row, nonce selector, mandatory expiry, atomic rotation (the
   old link dies in the same UPDATE that mints the fresh one), and the
   principal's status is re-checked on EVERY verification — a revoked
   principal's live-looking bearer refuses.
2. **Per-recipient HMAC** `{user_id}.{email}.{exp}.{mac}` under
   `PORTAL_RECIPIENT_TOKEN_SECRET` (domain-separated from the bearer
   secret). Stateless by design: verifiable from the link alone, no DB
   roundtrip; its mint audit row is its only durable trace. Rotating
   it = minting a fresh link; the old one lives out its own TTL.

Both MACs are verified constant-time. Every credential refusal shares
ONE body (`credential refused` / `portal_credential_refused`) — there
is no oracle distinguishing unknown, forged, expired, rotated, or
revoked.

## Signup is a declared policy, default OFF

`portal_signup_policies` ships EMPTY — zero install-time bootstrap. An
absent row reads CLOSED; only an explicit officer flip opens it, every
flip is audited (`policy_changed`), and the kill bites the very next
attempt. While closed, access is invitation-only: an invite is itself
a Tier A capability bound to the recipient's email, redeemable exactly
once, with an explicit revocation list and expiry.

## The surface downstream modules consume

`PortalDocumentSurface` (in
`src/application/service/portal_surface.rs`) is the declared contract:
ownership-scoped read models (`my_details`, `my_access_history`) and
one write verb (`update_my_details`) whose field whitelist is
`WRITABLE_DETAIL_FIELDS` — ten fields, declared once, enforced at the
SQL builder. No sudo-browse exists anywhere on the surface. The HTTP
router (`portal_public_routes()`) is a thin adapter over the same
verbs; hosts mount it at `/api/v1/portal`.

## Composing (host side)

```rust
use backbone_portal::presentation::http::public_routes::{
    portal_public_routes, PortalPublicState,
};

let state = PortalPublicState::from_env(pool);          // reads both *_TOKEN_SECRET vars
state.credential_slot().install(sapiens_verifier);       // sapiens owns password hashes
// unwired slot = every login/signup refuses LOUDLY (503 + audit), never silently

let app = Router::new()
    .nest("/api/v1/portal", portal_public_routes(state))
    .layer(middleware::from_fn(bearer_auth_middleware));

bus.register_handler(
    Arc::new(SapiensLifecycleHandler::new(pool.clone(), access))
).await?;
```

Required env (declare in the host's `.env.*.example`):
`PORTAL_BEARER_TOKEN_SECRET`, `PORTAL_RECIPIENT_TOKEN_SECRET`. An
unset secret is never a fallback: mints fail loudly with the typed
secret-not-configured error and verifies refuse.

## Testing

`cargo test` runs the fail-hard probe suite
(`tests/portal_probes.rs`): five classes — customer isolation (PISO),
credentials, invites, policy, lifecycle — each against its own
disposable scratch Postgres (default `localhost:5433`, override with
`PORTAL_TEST_ADMIN_URL`; NEVER a live service database). A skipped
probe is a failed probe. The generated HTTP integration tests
self-skip without a running server.

## Provenance

This module replaced a consuming-service skeleton (selling/billing/
support paths, framework v2.7.6). The skeleton's customer-isolation
test class was the one artifact carried forward — retargeted to this
module's own read models and kept mandatory. The predecessor is
preserved verbatim at git tag `v0.1.1`; the isolation ruling that
spawned the class is recorded in the serpa-workspace council records
(`docs/council/2026-07-09-service-portal-isolation.md`). The deeper
contract decisions live in [SPEC.md](SPEC.md).

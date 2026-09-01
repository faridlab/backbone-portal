# backbone-portal — the module contract

The durable record of what this module promises and why it is shaped
the way it is. The orientation lives in [README.md](README.md); this
document is the contract a host or a downstream website module can
hold the module to.

## Identity home and dependency posture

- Portal principals are stored in this module's own tables (schema
  `portal`). `portal_users` carries the ruling's address-block columns
  verbatim; `sapiens_user_id` is a LOGICAL reference to the sapiens
  account — no foreign key across module boundaries (the family rule:
  modules reference, hosts join).
- Dependency edges: `backbone-framework` (core/orm/auth/messaging) and
  `backbone-rate-limit`, all at tag `v2.7.11` — one rev, no mixing —
  plus exactly ONE module edge: `backbone-sapiens` at tag `v0.2.4`,
  taken for its published integration-event structs. No edge to
  sales-graph or website-family modules. Websites consume this
  module's trait surface; they are never its dependencies.
- Route naming follows the workspace ruling "module routes mount under
  schema names": the host nests the exported router at
  `/api/v1/portal`. The module itself mounts nothing.

## The two Tier A credentials (ADR-0018)

Independence is the load-bearing property: the recipient HMAC's MAC
input NEVER covers the bearer, and the bearer's input never covers the
recipient credential. Both directions are probe-asserted.

**Bearer capability** — `{token_id}.{nonce}.{exp}.{mac}`, MAC over
`(id, nonce, grant, exp)` under the bearer secret
(`PORTAL_BEARER_TOKEN_SECRET`). Semantics:

- `mac` is 64 hex chars, verified with a constant-time compare; the
  grant arm binds the row (the principal's status at verification
  time), so a revoked principal's still-unexpired bearer refuses.
- Verification recomputes the MAC over the STORED row fields — a link
  whose carried arms disagree with the row refuses.
- Rotation is one conditional UPDATE: the old row flips to `rotated`
  with `rotated_to` pointing at the fresh row in the same statement;
  there is no window where both verify.
- Default TTL 24 h (`DEFAULT_BEARER_TTL_HOURS`); expiry is mandatory.

**Per-recipient HMAC** — `{user_id}.{email}.{exp}.{mac}` under a
SEPARATE secret (`PORTAL_RECIPIENT_TOKEN_SECRET`), domain string
`portal-recipient`. Stateless: no row, verifiable from the link alone
in constant time; TTL 180 days; rotation = re-mint (the old link
lives out its own TTL — the digest-unsubscribe posture). The email may
itself contain dots, so the parser consumes arms from BOTH ends (id
first; mac then exp from the right; everything in the middle rejoined
is the email). A missing email arm leaves an empty middle whose MAC
cannot match anything minted.

**De-oracle rule** — every credential refusal, on every verb, shares
ONE body: `{"error":"credential refused","code":"portal_credential_refused"}`.
Unknown email, wrong password, revoked status, expired, rotated,
forged — indistinguishable from the outside. Audit rows record the
fact of refusal, never which half failed.

## Tier B throttle (the access verbs)

`AttemptBook` keys failures TWO ways — per normalized identity and per
IP. Curve (pure, probe-asserted): 5 consecutive failures lock for 30 s,
doubling per extra failure, capped at 15 min; 1 s minimum spacing
between attempts (anti-hammering); a success resets the key. The book
is in-memory per composing service — the accepted family trade: a
multi-instance host fronts the verbs with a shared limiter if it runs
more than one replica.

## Signup policy (kill switch)

`portal_signup_policies` ships EMPTY. An absent row reads CLOSED; a
disabled row reads CLOSED; only an explicit `enabled = true` opens
signup. Zero install-time bootstrap — a seeded-on row would be a
defect (probe-asserted: `signup_is_off_by_default_and_fail_closed`).
Every flip is audited (`policy_changed`), and the switch is re-read on
every attempt: a kill bites the very next request. Per-website opt-in
(if ever taken) is a HOST composition concern — the module's policy is
single and global; a website-scoped gate composes above the trait, not
inside it.

## Invitations (the only open path while closed)

An invite is a Tier A capability: `{invite_id}.{nonce}.{exp}.{mac}`
with grant `portal_invite:{recipient_email}` under the bearer secret.
- Bound to the recipient mailbox — a forwarded link refuses at
  redemption (probe: `forwarded_link_refuses`).
- ONE-SHOT: redemption is a conditional UPDATE to `redeemed`; a second
  presentation refuses (probe: `invite_redeems_exactly_once`).
- Explicit revocation list: `revoke_invite` (officer) and
  `revoke_pending_for_email` (the login-kill trigger) both audit.
- Expiry is re-derived from the STORED row at redemption; the
  `sweep_expired` verb only records terminal state — safety never
  depends on the sweep running.

## The declared surface (what websites consume)

`PortalDocumentSurface` — the trait contract, not route handlers:

- `my_details(principal)` → `PortalAccountView` — the caller's own
  record, scoped by the VERIFIED principal (the bearer IS the
  identity; no request-carried id exists on any read path).
- `my_access_history(principal, limit)` — the caller's own audit
  trail.
- `update_my_details(principal, patch)` → the one write verb.
  `WRITABLE_DETAIL_FIELDS` is the single declared whitelist (10
  fields), enforced at the QueryBuilder — unknown keys are dropped at
  the edge AND at the builder; the write stamps the acting principal
  into `metadata.updated_by`; an empty patch refuses; over-length
  values refuse with nothing written.

**The address-block whitelist decision record** — the ruling's
12-field portal allowlist maps to 10 writable columns:

| ruling field | column | note |
|---|---|---|
| name | `display_name` | |
| phone | `phone` | |
| street, street2 | `street`, `street2` | |
| city | `city` | |
| state, country | `state_id`, `country_id` | id references |
| zip, zipcode | `zip` | ONE column — the two ruling names are the same datum; carrying both would be a typo fork |
| vat | `vat` | |
| company | `company_name` | |
| email | — | NOT self-service writable: both credentials bind to the email; an email change is a re-invitation, not a PATCH |

**Company fence (ADR-0014), declared honestly**: this module declares
NO company fence. Portal identity is per-principal; the
customer/principal-per-company question belongs to the downstream
document modules that own documents. `company_name`/`vat` above are
display data the principal maintains about themselves, not a fence.

**The credential port (fail-closed)** — password hashes live in
sapiens, so verification is a port: `PortalCredentialVerifier` +
`CredentialVerifierSlot`, deny-by-default. An unwired slot makes every
login/signup refuse LOUDLY — typed `CredentialPortNotComposed` (503),
audited — never a silent allow and never a fake 401 (probe:
`unwired_credential_port_refuses_loudly`).

## The sapiens lifecycle subscription

`SapiensLifecycleHandler` subscribes on the HOST's bus (the host
registers it at compose time). Contract, verified first-hand against
the published v0.2.4 tag:

| event type | payload (published struct) | portal arm |
|---|---|---|
| `sapiens.user.created` | `user_id`, `email` (+ names) | conditional `sapiens_user_id` stamp on the matching principal; audited `onboarding_linked`; no principal = audited `onboarding_no_invitation` (a normal employee — never a silent skip) |
| `sapiens.user.deactivated` | `user_id` | revoke portal access + kill every live bearer + pending invites; audited `lifecycle_access_revoked` |
| `sapiens.user.deleted` | `user_id` | same revocation arm (deletion is terminal) |

- Both arms are re-delivery safe: the onboarding stamp is conditional
  (`sapiens_user_id IS NULL`); the revocation flip is conditional
  (`status <> 'revoked'`), so replays perform nothing and audit
  nothing new.
- A deactivated/deleted sapiens user with NO portal principal is a
  quiet no-op — most sapiens users are employees; auditing every
  employee deletion would bury the signal.
- The dependency is LOAD-BEARING at compile time: the handler derives
  its wire facts from the published
  `UserCreatedIntegrationEvent`/`UserDeletedIntegrationEvent` structs,
  so if the export moves, this crate stops compiling loudly instead of
  drifting.
- Producer gap, recorded honestly: at v0.2.4 only
  `sapiens.user.created` has live producers; the deactivate/delete
  producers land with the sapiens increment. The arms above are
  exercised with fixture envelopes and light up unchanged when the
  producers exist host-side.

## Mount and grants posture

- `portal_public_routes(state)` is a pure export; the host nests it at
  `/api/v1/portal`. Verbs: `POST /auth/redeem-invite`, `POST
  /auth/login`, `POST /auth/signup`, `POST /auth/rotate`, `GET /me`,
  `PATCH /me`, `GET /me/access-history`. Nothing is ever registered
  under `/api/v1/auth` — that path is the host's auth bridge.
- Migrations carry NO GRANTs (family precedent — module DDL is
  owner-role). The composing host runs its `rls_app_role.sql` after
  migrations to grant the schema to the app role; a host that skips it
  gets "permission denied for schema portal", by design.

## Audit vocabulary

`portal_audit_event` (append-only): `invite_minted`,
`invite_revoked`, `invite_redeemed`, `signup_refused_policy_closed`,
`signup_created`, `policy_changed`, `bearer_minted`, `bearer_rotated`,
`bearer_revoked`, `recipient_hmac_minted`, `login_succeeded`,
`login_refused`, `lifecycle_access_revoked`, `onboarding_linked`,
`onboarding_no_invitation`, `credential_port_not_composed`.

## Testing discipline

The probe suite (`tests/portal_probes.rs`) is fail-hard: one
disposable scratch Postgres per test (`localhost:5433` default,
`PORTAL_TEST_ADMIN_URL` override — never a live service database), raw
SQL migration runner, and `skipped()` PANICS — a skipped probe is a
failed probe. Classes: `piso_isolation` (customer isolation —
MANDATORY, the class carried forward from the module's
consuming-service predecessor at tag `v0.1.1`), `credentials`,
`invites`, `policy`, `lifecycle`.

Proven-by-revert is the isolation class's heritage: the honest way to
prove an ownership scope is to delete it, watch the probe go red,
restore it, and watch it go green. The probes are written so that
removing any `WHERE id = principal.user_id` arm, the identity stamp on
the write verb, or a status gate on a read turns them red.

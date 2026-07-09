# backbone-portal — a consuming service (not a domain module)

Thin-channels pillar (Tier 5) · **customer/supplier self-service** · posts no GL · owns no schema.

## What it is
A **consuming service**, per the plan's explicit call (tier5-deferred §5: *"a read-model + thin write
surface over selling/billing/support — built as a consuming service, not a new domain module, so it may
never need its own bounded context"*). It has **no schema, no entities, no bounded context**. It composes
the public contracts of `backbone-selling`, `backbone-billing`, and `backbone-support` into per-customer
read-models + a thin write surface. Because it is the composer, it **depends on** those modules directly —
that is not a horizontal module edge (those are forbidden between domain modules); it is the app assembling
views.

## The load-bearing invariant: customer data isolation
A portal user is ONE customer. **Every read is scoped to the authenticated `customer_id`** — a customer
must never see another customer's orders, invoices, or tickets. The `customer_id` is the session identity,
applied server-side; it is never a client-supplied filter the caller can widen. The write surface stamps
the same `customer_id` onto what it creates. This is the one thing a self-service portal can get
catastrophically wrong (a cross-customer data leak), so it is the focus of the review + the tests.

## Surface (`PortalService`)
- **Read-models** (each scoped to `customer_id`):
  - `my_orders(customer_id)` → the customer's sales orders (over `selling.sales_orders`)
  - `my_invoices(customer_id)` → the customer's invoices (over `billing.sales_invoices`)
  - `my_tickets(customer_id)` → the customer's support tickets (over `support.issues`)
- **Write surface:**
  - `submit_ticket(customer_id, company_id, subject, description)` → drives the REAL
    `support.raise_issue` (the `customer_id` is stamped from the session)

## Tests
`tests/portal_isolation.rs` (2, composing the REAL selling/billing/support schemas + the REAL support write
path):
- **PISO-1 — reads are customer-scoped.** Customer A never sees B's orders/invoices/tickets, and nothing of
  A's leaks into B's views. **Proven-by-revert:** dropping the `customer_id` scope on a read leaks another
  customer's rows (PISO-1 red); the scope restored → green.
- **PISO-2 — the write surface stamps the session customer.** A submitted ticket lands as a REAL support
  issue scoped to the submitting customer; a blank subject is refused.

Security review record: `docs/council/2026-07-09-service-portal-isolation.md`.

## Deferred (with reason)
The HTTP/gRPC server binary + session auth middleware (composition boilerplate a deployment adds — the
`customer_id` here stands in for the authenticated session), supplier-side views, RFQ submission, real-time
order tracking. Promoted against a named self-service requirement (tier5-deferred §5).

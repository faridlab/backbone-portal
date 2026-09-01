-- Hardening constraints the schema DSL cannot express (hand-written;
-- user-owned — see metaphor.codegen.yaml).
--
-- 1. Unique-on-live email: at most ONE live principal per email. The
--    soft-delete convention (metadata->>'deleted_at') means a plain
--    UNIQUE constraint would forbid re-inviting a archived principal's
--    address forever; the partial index keeps the race fence (PI-07)
--    among LIVE rows only. Signup's ON CONFLICT fallback and invite
--    redemption both rely on this fence being the database's, not a
--    check-then-insert search.
-- 2. Signup-policy singleton: at most ONE policy row EVER. The
--    policy read is fail-closed (absent row = closed), so a second row
--    could only ever shadow the deliberate one; the constant-expression
--    index makes the table a singleton by construction. History lives
--    in the audit log (policy_changed rows), not in dead rows.

CREATE UNIQUE INDEX IF NOT EXISTS idx_portal_users_email_live
    ON portal.portal_users (email)
    WHERE (metadata->>'deleted_at') IS NULL;

CREATE UNIQUE INDEX IF NOT EXISTS idx_portal_signup_policies_singleton
    ON portal.portal_signup_policies ((1));

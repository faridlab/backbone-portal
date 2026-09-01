-- Down migration for the portal hardening constraints.

DROP INDEX IF EXISTS portal.idx_portal_signup_policies_singleton;
DROP INDEX IF EXISTS portal.idx_portal_users_email_live;

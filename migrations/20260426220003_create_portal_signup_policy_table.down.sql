-- Down: drop portal.portal_signup_policies table
DROP TABLE IF EXISTS portal.portal_signup_policies CASCADE;
DROP FUNCTION IF EXISTS portal.portal_signup_policies_audit_timestamp() CASCADE;

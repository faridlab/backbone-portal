-- Down: drop portal.portal_users table
DROP TABLE IF EXISTS portal.portal_users CASCADE;
DROP FUNCTION IF EXISTS portal.portal_users_audit_timestamp() CASCADE;

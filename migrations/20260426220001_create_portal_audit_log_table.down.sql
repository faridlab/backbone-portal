-- Down: drop portal.portal_audit_log table
DROP TABLE IF EXISTS portal.portal_audit_log CASCADE;
DROP FUNCTION IF EXISTS portal.portal_audit_log_audit_timestamp() CASCADE;

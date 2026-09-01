-- Down: drop portal.portal_tokens table
DROP TABLE IF EXISTS portal.portal_tokens CASCADE;
DROP FUNCTION IF EXISTS portal.portal_tokens_audit_timestamp() CASCADE;

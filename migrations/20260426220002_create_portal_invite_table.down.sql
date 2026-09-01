-- Down: drop portal.portal_invites table
DROP TABLE IF EXISTS portal.portal_invites CASCADE;
DROP FUNCTION IF EXISTS portal.portal_invites_audit_timestamp() CASCADE;

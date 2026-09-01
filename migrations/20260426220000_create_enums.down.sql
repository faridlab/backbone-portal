-- Down: drop enum types for portal module
DROP TYPE IF EXISTS portal_user_status CASCADE;
DROP TYPE IF EXISTS portal_token_status CASCADE;
DROP TYPE IF EXISTS portal_invite_status CASCADE;
DROP TYPE IF EXISTS portal_audit_event CASCADE;

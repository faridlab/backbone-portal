-- Down: drop the declared foreign keys for portal module
ALTER TABLE IF EXISTS portal.portal_tokens DROP CONSTRAINT IF EXISTS fk_portal_tokens_user_id;

-- The de-duplicating UPDATE/DELETEs in up.sql are not reversible; only the
-- constraints and indexes they enabled are dropped here.
DROP INDEX IF EXISTS "website_user_id_idx";
DROP INDEX IF EXISTS "website_tick_website_created_idx";
DROP INDEX IF EXISTS "channel_link_pairing_code_key";
DROP INDEX IF EXISTS "user_username_key";

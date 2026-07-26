-- Uniqueness that was only ever enforced in application code, plus the
-- indexes the hot read paths assume exist.

-- 1. Usernames.
--
-- Signup checked for an existing username with a SELECT and then INSERTed,
-- so two concurrent signups could both pass the check. `get_user_by_username`
-- resolves a login with `.first()`, meaning a duplicate makes sign-in land on
-- an arbitrary one of the two accounts.
--
-- Existing duplicates (if any) are already broken accounts today: only one of
-- them is reachable. Rather than delete anyone's data, the later row is
-- renamed so the index can be created, and the owner can sort it out.
UPDATE "user" AS u
SET username = u.username || '~dup~' || left(u.id, 8)
WHERE EXISTS (
    SELECT 1
    FROM "user" AS other
    WHERE other.username = u.username
      AND other.id < u.id
);

CREATE UNIQUE INDEX "user_username_key" ON "user" ("username");

-- 2. Channel pairing codes.
--
-- A pairing code is how a chat proves which pending link it is, and
-- `approve_channel_link` resolves exactly one row from it. Codes are 6 hex
-- characters, so collisions are not negligible: two pending links sharing one
-- made that query fail outright, and meant a code could confirm a chat other
-- than the one it was issued to.
--
-- Still-pending duplicates are simply removed - the next message from that
-- chat recreates the link with a fresh code. Already-linked rows keep their
-- row, with the (now meaningless) code replaced by their own id.
DELETE FROM "channel_link" AS a
USING "channel_link" AS b
WHERE a.pairing_code = b.pairing_code
  AND a.id > b.id
  AND a.user_id IS NULL;

UPDATE "channel_link" AS c
SET pairing_code = c.id
WHERE EXISTS (
    SELECT 1
    FROM "channel_link" AS other
    WHERE other.pairing_code = c.pairing_code
      AND other.id < c.id
);

CREATE UNIQUE INDEX "channel_link_pairing_code_key"
    ON "channel_link" ("pairing_code");

-- 3. Read-path indexes.
--
-- website_tick grows by one row per monitor per check, forever. Every monitor
-- view ("latest 20 ticks", "all ticks since T", "latest tick") filters by
-- website_id and orders by createdAt, so without this index each one is a
-- sequential scan of the entire table - including the unauthenticated public
-- status page, which reads a 30-day window per published monitor.
CREATE INDEX "website_tick_website_created_idx"
    ON "website_tick" ("website_id", "createdAt" DESC);

CREATE INDEX "website_user_id_idx" ON "website" ("user_id");

-- Your SQL goes here
CREATE TABLE "channel_link" (
    "id" TEXT PRIMARY KEY,
    "channel" TEXT NOT NULL,
    "channel_user_id" TEXT NOT NULL,
    "user_id" TEXT REFERENCES "user" ("id") ON DELETE CASCADE,
    "pairing_code" TEXT NOT NULL,
    "history" TEXT NOT NULL DEFAULT '[]',
    "created_at" TIMESTAMP NOT NULL DEFAULT now(),
    "updated_at" TIMESTAMP NOT NULL DEFAULT now(),
    UNIQUE ("channel", "channel_user_id")
);

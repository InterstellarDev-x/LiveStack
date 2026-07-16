-- Your SQL goes here
CREATE TABLE "website_notification_config" (
    "website_id" TEXT PRIMARY KEY REFERENCES "website" ("id") ON DELETE CASCADE,
    "webhook_url" TEXT,
    "webhook_secret" TEXT,
    "created_at" TIMESTAMP NOT NULL DEFAULT now(),
    "updated_at" TIMESTAMP NOT NULL DEFAULT now()
);

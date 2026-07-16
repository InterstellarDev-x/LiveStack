-- Your SQL goes here
ALTER TABLE "website_notification_config"
    ADD COLUMN "webhook_enabled" BOOLEAN NOT NULL DEFAULT true;

ALTER TABLE "user"
    ADD COLUMN "email_alerts_enabled" BOOLEAN NOT NULL DEFAULT true;

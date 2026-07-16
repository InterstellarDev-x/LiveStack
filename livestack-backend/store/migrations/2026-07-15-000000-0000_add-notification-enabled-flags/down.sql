-- This file should undo anything in `up.sql`
ALTER TABLE "website_notification_config"
    DROP COLUMN "webhook_enabled";

ALTER TABLE "user"
    DROP COLUMN "email_alerts_enabled";

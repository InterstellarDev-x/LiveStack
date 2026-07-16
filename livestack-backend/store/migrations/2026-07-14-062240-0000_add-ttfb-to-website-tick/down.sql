-- This file should undo anything in `up.sql`
ALTER TABLE "website_tick"
    DROP COLUMN "waiting_time_ms";

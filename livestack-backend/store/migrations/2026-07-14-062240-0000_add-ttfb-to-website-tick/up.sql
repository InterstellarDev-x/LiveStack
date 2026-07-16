-- Your SQL goes here
ALTER TABLE "website_tick"
    ADD COLUMN "waiting_time_ms" INTEGER NOT NULL DEFAULT 0;

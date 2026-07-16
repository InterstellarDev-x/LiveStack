-- Your SQL goes here
ALTER TABLE "website_tick"
    ADD COLUMN "dns_time_ms" INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN "connection_time_ms" INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN "tls_time_ms" INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN "data_transfer_time_ms" INTEGER NOT NULL DEFAULT 0;

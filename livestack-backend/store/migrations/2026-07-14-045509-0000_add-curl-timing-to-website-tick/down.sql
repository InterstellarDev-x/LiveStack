-- This file should undo anything in `up.sql`
ALTER TABLE "website_tick"
    DROP COLUMN "dns_time_ms",
    DROP COLUMN "connection_time_ms",
    DROP COLUMN "tls_time_ms",
    DROP COLUMN "data_transfer_time_ms";

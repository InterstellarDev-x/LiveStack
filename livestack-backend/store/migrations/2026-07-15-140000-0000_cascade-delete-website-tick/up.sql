-- Your SQL goes here
ALTER TABLE "website_tick" DROP CONSTRAINT "website_tick_website_id_fkey";
ALTER TABLE "website_tick" ADD CONSTRAINT "website_tick_website_id_fkey"
    FOREIGN KEY ("website_id") REFERENCES "website"("id") ON DELETE CASCADE ON UPDATE CASCADE;

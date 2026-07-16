-- Your SQL goes here
CREATE TABLE "status_page" (
    "id" TEXT PRIMARY KEY,
    "user_id" TEXT NOT NULL REFERENCES "user" ("id") ON DELETE CASCADE,
    "slug" TEXT NOT NULL UNIQUE,
    "title" TEXT NOT NULL,
    "created_at" TIMESTAMP NOT NULL DEFAULT now(),
    "updated_at" TIMESTAMP NOT NULL DEFAULT now()
);

CREATE TABLE "status_page_monitor" (
    "id" TEXT PRIMARY KEY,
    "status_page_id" TEXT NOT NULL REFERENCES "status_page" ("id") ON DELETE CASCADE,
    "website_id" TEXT NOT NULL REFERENCES "website" ("id") ON DELETE CASCADE,
    "display_name" TEXT NOT NULL,
    "sort_order" INT4 NOT NULL DEFAULT 0,
    UNIQUE ("status_page_id", "website_id")
);

-- Your SQL goes here
CREATE TABLE "incident" (
    "id" TEXT PRIMARY KEY,
    "website_id" TEXT NOT NULL REFERENCES "website" ("id") ON DELETE CASCADE,
    "started_at" TIMESTAMP NOT NULL DEFAULT now(),
    "resolved_at" TIMESTAMP,
    "cause" TEXT NOT NULL
);

-- At most one open (unresolved) incident per website. This is what makes the
-- consumer's open/resolve state machine race-safe across multiple workers:
-- whichever INSERT lands owns the "incident opened" transition.
CREATE UNIQUE INDEX "one_open_incident_per_website"
    ON "incident" ("website_id") WHERE "resolved_at" IS NULL;

-- History reads are always "newest incidents for this website".
CREATE INDEX "incident_website_started_at_idx"
    ON "incident" ("website_id", "started_at" DESC);

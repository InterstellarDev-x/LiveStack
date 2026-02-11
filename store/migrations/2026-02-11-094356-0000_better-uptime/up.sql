-- Your SQL goes here




CREATE TABLE "user" (
    "id" TEXT NOT NULL,
    "email" TEXT NOT NULL,
    "password" TEXT NOT NUll
)


CREATE TABLE "website" (
    "id" TEXT NOT NULL,
    "url" TEXT NOT NULL,
    "time_added" TIMESTAMP(3) NOT NULL
)

CREATE TABLE "region" (
    "id" TEXT NOT NULL,
    "name" TEXT NOT NULL

    CONSTRAINT "region_pkey" PRIMARY KEY ("id")
)


CREATE TABLE "website_tick" (
    "id" TEXT NOT NULL,
    "response_time_ms" INTEGER NOT NULL
    
)
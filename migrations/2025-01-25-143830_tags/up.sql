-- Your SQL goes here

CREATE TYPE "tag_priority" AS ENUM ('HIGH', 'MEDIUM', 'LOW', 'HIDE');

CREATE TABLE "tags"(
	"key" VARCHAR NOT NULL PRIMARY KEY,
	"name" VARCHAR NOT NULL,
	"description" VARCHAR NOT NULL,
	"short_name" VARCHAR,
	"priority" tag_priority DEFAULT 'HIDE' NOT NULL,
	"is_allergy" BOOL DEFAULT false NOT NULL
);


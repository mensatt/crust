-- Your SQL goes here


CREATE TABLE "locations"(
	"id" UUID NOT NULL PRIMARY KEY,
	"external_id" BIGINT NOT NULL,
	"name" VARCHAR NOT NULL,
	"visible" BOOL DEFAULT false NOT NULL
);


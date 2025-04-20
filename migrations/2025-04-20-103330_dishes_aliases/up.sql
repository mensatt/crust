-- Your SQL goes here

CREATE TABLE "dishes_aliases" (
    alias_name VARCHAR NOT NULL PRIMARY KEY,
    normalized_alias_name VARCHAR NOT NULL,
    dish UUID NOT NULL REFERENCES dishes(id) ON DELETE CASCADE
);

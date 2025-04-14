-- Your SQL goes here

CREATE TABLE "reviews" (
    id UUID NOT NULL PRIMARY KEY,
    display_name VARCHAR,
    stars BIGINT NOT NULL,
    text VARCHAR,
    created_at TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL,
    accepted_at TIMESTAMPTZ,
    occurrence UUID NOT NULL REFERENCES occurrences(id) ON DELETE CASCADE
);

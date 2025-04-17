-- Your SQL goes here

CREATE TABLE "images" (
    id UUID NOT NULL PRIMARY KEY,
    review UUID NOT NULL REFERENCES reviews(id) ON DELETE CASCADE
);

-- Your SQL goes here

CREATE TABLE "occurrences_tags" (
    occurrence UUID REFERENCES occurrences(id) ON DELETE CASCADE,
    tag VARCHAR REFERENCES tags(key) ON DELETE CASCADE,
    PRIMARY KEY (occurrence, tag)
);

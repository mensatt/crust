-- Your SQL goes here

CREATE TABLE "occurrences_side_dishes" (
    occurrence UUID REFERENCES occurrences(id) ON DELETE CASCADE,
    dish UUID REFERENCES dishes(id) ON DELETE CASCADE,
    PRIMARY KEY (occurrence, dish)
);

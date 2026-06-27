-- This file should undo anything in `up.sql`

DROP INDEX IF EXISTS idx_images_review;
DROP INDEX IF EXISTS idx_dishes_aliases_dish;
DROP INDEX IF EXISTS idx_occurrences_dish;
DROP INDEX IF EXISTS idx_occurrences_location_date;
DROP INDEX IF EXISTS idx_reviews_occurrence;

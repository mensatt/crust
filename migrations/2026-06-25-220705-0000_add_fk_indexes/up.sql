-- Plain FK indexes
CREATE INDEX idx_images_review ON images (review);
CREATE INDEX idx_dishes_aliases_dish ON dishes_aliases (dish);
CREATE INDEX idx_occurrences_dish ON occurrences (dish);

-- occurrences() often filters by location and date
CREATE INDEX idx_occurrences_location_date ON occurrences (location, date);

-- ReviewLoader / review images join filter by occurrence
-- INCLUDE (stars) lets review_data.metadata's avg(stars)/count(id) run as an index-only scan
CREATE INDEX idx_reviews_occurrence ON reviews (occurrence) INCLUDE (stars);

-- Remove constraint that ensures german dish names are unique
ALTER TABLE dishes DROP CONSTRAINT dishes_name_de_key;

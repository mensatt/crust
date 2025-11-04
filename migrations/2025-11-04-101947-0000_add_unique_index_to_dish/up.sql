-- Add constraint to ensure german dish names are unique
ALTER TABLE dishes ADD CONSTRAINT dishes_name_de_key UNIQUE (name_de);

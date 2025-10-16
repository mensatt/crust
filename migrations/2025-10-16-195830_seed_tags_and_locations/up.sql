-- Your SQL goes here

-- Insert Tags
-- Meat/Protein Tags (HIGH priority)
INSERT INTO tags (key, name, description, short_name, priority, is_allergy) VALUES
('S', 'Schwein', 'Schweinefleisch', '🐷', 'HIGH', false),
('R', 'Rind', 'Rindfleisch', '🐮', 'HIGH', false),
('G', 'Geflügel', 'Geflügelfleisch', '🐔', 'HIGH', false),
('L', 'Lamm', 'Lammfleisch', '🐑', 'HIGH', false),
('W', 'Wild', 'Wildfleisch', '🦌', 'HIGH', false),
('F', 'Fisch', 'Fisch', '🐟', 'HIGH', false),
('V', 'Vegetarisch', 'Vegetarisches Gericht', '🥕', 'HIGH', false),
('Veg', 'Vegan', 'Veganes Gericht', '🌱', 'HIGH', false),
('MSC', 'MSC Fisch', 'MSC-zertifizierter Fisch', '🐟', 'HIGH', false);

-- Certification/Lifestyle Tags (MEDIUM priority)
INSERT INTO tags (key, name, description, short_name, priority, is_allergy) VALUES
('Gf', 'Glutenfrei', 'Glutenfreies Gericht', NULL, 'MEDIUM', true),
('CO2', 'CO2-Neutral', 'CO2-neutrales Gericht', NULL, 'MEDIUM', false),
('Bio', 'Bio', 'Biologisches Gericht', NULL, 'MEDIUM', false),
('MV', 'MensaVital', 'MensaVital Gericht', NULL, 'MEDIUM', false);

-- Allergen Tags (Gluten group - MEDIUM priority)
INSERT INTO tags (key, name, description, short_name, priority, is_allergy) VALUES
('Wz', 'Weizen', 'enthält Weizen (Gluten)', '🌾', 'MEDIUM', true),
('Ro', 'Roggen', 'enthält Roggen (Gluten)', NULL, 'MEDIUM', true),
('Ge', 'Gerste', 'enthält Gerste (Gluten)', NULL, 'MEDIUM', true),
('Hf', 'Hafer', 'enthält Hafer (Gluten)', NULL, 'MEDIUM', true);

-- Allergen Tags (Other common allergens - MEDIUM priority)
INSERT INTO tags (key, name, description, short_name, priority, is_allergy) VALUES
('Er', 'Erdnüsse', 'enthält Erdnüsse', '🥜', 'MEDIUM', true),
('Mi', 'Milch/Laktose', 'enthält Milch/Laktose', '🥛', 'MEDIUM', true),
('A', 'Alkohol', 'enthält Alkohol', '🍺', 'MEDIUM', true);

-- Allergen Tags (LOW priority)
INSERT INTO tags (key, name, description, short_name, priority, is_allergy) VALUES
('Kr', 'Krebstiere', 'enthält Krebstiere', '🦀', 'LOW', true),
('Ei', 'Eier', 'enthält Eier', '🥚', 'LOW', true),
('Fi', 'Fisch', 'enthält Fisch', '🐟', 'LOW', true),
('So', 'Soja', 'enthält Soja', NULL, 'LOW', true),
('Man', 'Mandeln', 'enthält Mandeln', NULL, 'LOW', true),
('Hs', 'Haselnüsse', 'enthält Haselnüsse', '🌰', 'LOW', true),
('Wa', 'Walnüsse', 'enthält Walnüsse', NULL, 'LOW', true),
('Ka', 'Cashewnüsse', 'enthält Cashewnüsse', NULL, 'LOW', true),
('Pe', 'Pekanüsse', 'enthält Pekanüsse', NULL, 'LOW', true),
('Pa', 'Paranüsse', 'enthält Paranüsse', NULL, 'LOW', true),
('Pi', 'Pistazien', 'enthält Pistazien', NULL, 'LOW', true),
('Mac', 'Macadamianüsse', 'enthält Macadamianüsse', NULL, 'LOW', true),
('Sel', 'Sellerie', 'enthält Sellerie', NULL, 'LOW', true),
('Sen', 'Senf', 'enthält Senf', NULL, 'LOW', true),
('Ses', 'Sesam', 'enthält Sesam', NULL, 'LOW', true),
('Su', 'Schwefeloxid/Sulfite', 'enthält Schwefeloxid/Sulfite', NULL, 'LOW', true),
('Lu', 'Lupinen', 'enthält Lupinen', NULL, 'LOW', true),
('We', 'Weichtiere', 'enthält Weichtiere', NULL, 'LOW', true);

-- Additive/Ingredient Information Tags (HIDE priority)
INSERT INTO tags (key, name, description, short_name, priority, is_allergy) VALUES
('1', 'Farbstoff', 'mit Farbstoff', '🎨', 'HIDE', false),
('2', 'Koffein', 'mit Koffein', '☕', 'HIDE', false),
('4', 'Konservierungsstoffe', 'mit Konservierungsstoffen', NULL, 'HIDE', false),
('5', 'Süßungsmittel', 'mit Süßungsmittel', NULL, 'HIDE', false),
('7', 'Antioxidationsmittel', 'mit Antioxidationsmittel', NULL, 'HIDE', false),
('8', 'Geschmacksverstärker', 'mit Geschmacksverstärker', NULL, 'HIDE', false),
('9', 'geschwefelt', 'geschwefelt', NULL, 'HIDE', false),
('10', 'geschwärzt', 'geschwärzt', NULL, 'HIDE', false),
('11', 'gewachst', 'gewachst', NULL, 'HIDE', false),
('12', 'Phosphat', 'mit Phosphat', NULL, 'HIDE', false),
('13', 'Phenylalaninquelle', 'mit Phenylalaninquelle', NULL, 'HIDE', false),
('30', 'Fettglasur', 'mit Fettglasur', NULL, 'HIDE', false);

-- Insert Locations
INSERT INTO locations (id, external_id, name, visible) VALUES
('eddfa64d-5f21-4515-97d4-d45e49168116', 1, 'Erlangen Südmensa', true),
('5323310f-71c5-47a2-be71-f6b4e2619a86', 2, 'Nürnberg Insel Schütt', true),
('9cd246b3-5e1b-40f4-bffc-5bf31480c8cf', 3, 'Nürnberg Regenburger Straße', true),
('83c942f0-7c64-44a2-aeca-925a28d4325d', 4, 'Ansbach', false),
('dfb91b0f-7ce4-43c0-a9d7-c6b3016cfd85', 5, 'Eichstätt', false),
('98754fad-1af8-492f-b800-33a1bf5a4a05', 6, 'Nürnberg Ohm', true),
('e1a6e00b-2a00-461d-9afd-b2e53fd264f0', 7, 'Ingolstadt', true),
('89812062-d3e6-4b2e-abe8-bd8d561aebae', 8, 'Erlangen Langemarckplatz', true),
('ec52fb85-6dce-403d-9c89-fafebbff4610', 9, 'Nürnberg St. Paul', true),
('38da9889-946d-4a52-a65c-9f867b328835', 12, 'Triesdorf', false)

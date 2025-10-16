-- This file should undo anything in `up.sql`

-- Delete all seeded locations
DELETE FROM locations WHERE id IN (
    'eddfa64d-5f21-4515-97d4-d45e49168116',
    '5323310f-71c5-47a2-be71-f6b4e2619a86',
    '9cd246b3-5e1b-40f4-bffc-5bf31480c8cf',
    '83c942f0-7c64-44a2-aeca-925a28d4325d',
    'dfb91b0f-7ce4-43c0-a9d7-c6b3016cfd85',
    '98754fad-1af8-492f-b800-33a1bf5a4a05',
    'e1a6e00b-2a00-461d-9afd-b2e53fd264f0',
    '89812062-d3e6-4b2e-abe8-bd8d561aebae',
    'ec52fb85-6dce-403d-9c89-fafebbff4610',
    '38da9889-946d-4a52-a65c-9f867b328835'
);

-- Delete all seeded tags
DELETE FROM tags WHERE key IN (
    'S', 'R', 'G', 'L', 'W', 'F', 'V', 'Veg', 'MSC',
    'Gf', 'CO2', 'Bio', 'MV',
    'Wz', 'Ro', 'Ge', 'Hf',
    'Er', 'Mi', 'A',
    'Kr', 'Ei', 'Fi', 'So', 'Man', 'Hs', 'Wa', 'Ka', 'Pe', 'Pa', 'Pi', 'Mac', 'Sel', 'Sen', 'Ses', 'Su', 'Lu', 'We',
    '1', '2', '4', '5', '7', '8', '9', '10', '11', '12', '13', '30'
)

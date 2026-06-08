-- Insert a group
INSERT INTO GROUPS(
  name,
  code,
  description,
  contribution_amount,
  frequency,
  next_payout_date,
  is_active,
  created_at
)
VALUES(
  'Makola Traders',
  'MKL',
  'Market women savings group',
  100,
  'weekly',
  '2026-07-15 00:00:00',
  TRUE,
  now()
);
-- Insert members for that group
INSERT INTO members(
  name,
  phone,
  trust_score,
  group_id,
  total_contributed,
  on_time_payments,
  total_payments,
  status,
  joined_at
)
VALUES(
  'Ama Mensah',
  '0241234567',
  100,
  1,
  2600,
  26,
  26,
  'active',
  now()
),
(
  'Adwoa Asante',
  '0242345678',
  85,
  1,
  2210,
  22,
  26,
  'active',
  now()
),
(
  'Akua Boateng',
  '0243456789',
  62,
  1,
  1560,
  16,
  26,
  'active',
  now()
);
-- Insert a second group
INSERT INTO GROUPS(
  name,
  code,
  description,
  contribution_amount,
  frequency,
  next_payout_date,
  is_active,
  created_at
)
VALUES(
  'Abossey Okai Spare Parts',
  'AOS',
  'Spare parts dealers',
  200,
  'monthly',
  '2026-06-05 00:00:00',
  TRUE,
  now()
);
-- Insert members for second group
INSERT INTO members(
  name,
  phone,
  trust_score,
  group_id,
  total_contributed,
  on_time_payments,
  total_payments,
  status,
  joined_at
)
VALUES(
  'Kwame Asante',
  '0201234567',
  100,
  2,
  6000,
  30,
  30,
  'active',
  now()
),
(
  'Kofi Boateng',
  '0202345678',
  73,
  2,
  4380,
  22,
  30,
  'active',
  now()
);
-- Update group member counts
UPDATE GROUPS
SET member_count = 3
WHERE
  id = 1;
UPDATE GROUPS
SET member_count = 2
WHERE
  id = 2;
-- Update group total contributions
UPDATE GROUPS
SET total_contributed = 6370
WHERE
  id = 1;
-- 2600 + 2210 + 1560
UPDATE GROUPS
SET total_contributed = 10380
WHERE
  id = 2;
-- 6000 + 4380

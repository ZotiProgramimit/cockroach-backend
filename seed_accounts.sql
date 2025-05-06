INSERT INTO accounts (id, username, balance)
SELECT
  ('00000000-0000-0000-0000-' || lpad(g::string, 12, '0'))::uuid,
  'user_' || lpad(g::string, 4, '0'),
  100000
FROM generate_series(0, 100000) AS g
ON CONFLICT ON CONSTRAINT accounts_username_key DO NOTHING;
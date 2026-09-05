SELECT
  id,
  (created_at + INTERVAL 7 day) AS due_date,
  date_diff('day', created_at, shipped_at) AS days_to_ship,
  COALESCE(note, 'none') AS note,
  amount::INT AS amount_int
FROM orders
WHERE (created_at + INTERVAL 30 day) >= created_at
ORDER BY id;

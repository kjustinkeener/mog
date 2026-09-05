SELECT
  id,
  (created_at + INTERVAL 7 DAY) AS due_date,
  date_diff('DAY', created_at, shipped_at) AS days_to_ship,
  TRY_CAST(amount AS BIGINT) AS amount_int,
  COALESCE(note, 'none') AS note
FROM orders
WHERE date_diff('DAY', created_at, shipped_at) > 3
ORDER BY id;

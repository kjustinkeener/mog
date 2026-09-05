SELECT
  id,
  (created_at + INTERVAL '7 day') AS due_date,
  DATEDIFF(day, created_at, shipped_at) AS days_to_ship, -- TODO(mog): Postgres has no part-aware DATEDIFF; for day diffs on dates use (end - start), else compute via EXTRACT / AGE
  COALESCE(note, 'none') AS note,
  amount::INT AS amount_int
FROM orders
WHERE (created_at + INTERVAL '30 day') >= created_at
ORDER BY id;

SELECT
  id,
  DATE_ADD(created_at, INTERVAL 7 DAY) AS due_date,
  DATE_DIFF(shipped_at, created_at, DAY) AS days_to_ship,
  SAFE_CAST(amount AS INT64) AS amount_int,
  IFNULL(note, 'none') AS note
FROM orders
WHERE DATE_DIFF(shipped_at, created_at, DAY) > 3
ORDER BY id;

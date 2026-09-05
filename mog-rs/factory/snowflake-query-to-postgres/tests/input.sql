SELECT
  id,
  DATEADD(day, 7, created_at) AS due_date,
  DATEDIFF(day, created_at, shipped_at) AS days_to_ship,
  NVL(note, 'none') AS note,
  amount::INT AS amount_int
FROM orders
WHERE DATEADD(day, 30, created_at) >= created_at
ORDER BY id;

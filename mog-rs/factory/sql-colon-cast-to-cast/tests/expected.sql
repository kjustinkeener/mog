SELECT
  CAST(amount AS NUMERIC(10,2)),
  CAST(order_id AS STRING),
  CAST(t.created_at AS DATE),
  CAST("created_at" AS DATE),
  CAST((a + b) AS INT)
FROM orders;

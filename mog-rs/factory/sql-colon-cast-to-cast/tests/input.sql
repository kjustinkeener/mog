SELECT
  amount::NUMERIC(10,2),
  order_id::STRING,
  t.created_at::DATE,
  "created_at"::DATE,
  (a + b)::INT
FROM orders;

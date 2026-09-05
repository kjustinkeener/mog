CREATE OR REPLACE TABLE "analytics"."orders" (
  "order_id" NUMBER(38,0),
  "customer_id" NUMBER,
  "amount" NUMBER(10,2),
  "created_at" TIMESTAMP_NTZ,
  "updated_at" TIMESTAMP_TZ,
  "notes" VARCHAR(500),
  "payload" VARIANT,
  "is_active" BOOLEAN
);

SELECT
  "order_id",
  NVL("amount", 0) AS amount,
  IFF("is_active", 1, 0) AS active_flag,
  "created_at"::DATE AS created_date
FROM "analytics"."orders";

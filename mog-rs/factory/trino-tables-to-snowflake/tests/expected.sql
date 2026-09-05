CREATE OR REPLACE TABLE customers (
  id BIGINT,
  name VARCHAR,
  balance DECIMAL(18,2),
  attrs VARIANT,
  created_at TIMESTAMP_NTZ,
  updated_at TIMESTAMP_TZ
);

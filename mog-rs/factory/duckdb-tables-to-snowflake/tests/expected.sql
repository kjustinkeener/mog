CREATE OR REPLACE TABLE events(
  id BIGINT,
  name VARCHAR,
  payload VARIANT,
  amount DECIMAL(18,2),
  created_at TIMESTAMP_NTZ,
  updated_at TIMESTAMP_TZ,
  tags ARRAY -- TODO(mog): DuckDB LIST/array mapped to Snowflake ARRAY (untyped); element type lost
);

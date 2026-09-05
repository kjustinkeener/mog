CREATE OR REPLACE TABLE main.sales.customers (
  id BIGINT,
  name VARCHAR,
  email VARCHAR,
  balance DECIMAL(18,2),
  tags ARRAY, -- TODO(mog): Snowflake ARRAY/OBJECT is untyped; the Databricks element/field types were dropped
  attrs OBJECT, -- TODO(mog): Snowflake ARRAY/OBJECT is untyped; the Databricks element/field types were dropped
  created_at TIMESTAMP_LTZ, -- NOTE(mog): mapped Databricks TIMESTAMP (UTC instant) to Snowflake TIMESTAMP_LTZ; use TIMESTAMP_NTZ instead if the source was wall-clock
  region VARCHAR)
-- TODO(mog): dropped Databricks PARTITIONED BY (region) (set target partitioning / clustering manually)
COMMENT = 'customer master table'
-- TODO(mog): dropped source LOCATION 's3://acme-lake/warehouse/sales.db/customers' (set target storage / external stage manually)

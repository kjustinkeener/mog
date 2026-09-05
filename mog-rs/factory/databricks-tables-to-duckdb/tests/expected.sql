CREATE TABLE customers (
  id BIGINT,
  name VARCHAR,
  email VARCHAR,
  balance DECIMAL(18,2),
  tags JSON, -- TODO(mog): collapsed from a Databricks nested type; element/field types were lost
  attrs JSON, -- TODO(mog): collapsed from a Databricks nested type; element/field types were lost
  created_at TIMESTAMPTZ,
  region VARCHAR)
-- TODO(mog): dropped Databricks PARTITIONED BY (region) (set target partitioning / clustering manually)
-- TODO(mog): table comment dropped (use COMMENT ON TABLE in DuckDB)
-- TODO(mog): dropped source LOCATION 's3://acme-lake/warehouse/sales.db/customers' (set target storage / external stage manually)

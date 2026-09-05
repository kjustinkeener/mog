CREATE TABLE sales.customers (
  id BIGINT,
  name TEXT,
  email TEXT,
  balance DECIMAL(18,2),
  tags JSONB, -- TODO(mog): collapsed from a Databricks nested type; element/field types were lost
  attrs JSONB, -- TODO(mog): collapsed from a Databricks nested type; element/field types were lost
  created_at TIMESTAMPTZ,
  region TEXT)
-- TODO(mog): dropped Databricks PARTITIONED BY (region) (set target partitioning / clustering manually)
-- TODO(mog): table comment dropped (use COMMENT ON TABLE in Postgres)
-- TODO(mog): dropped source LOCATION 's3://acme-lake/warehouse/sales.db/customers' (set target storage / external stage manually)

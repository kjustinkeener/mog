CREATE TABLE customers (
  id BIGINT,
  name VARCHAR(65535),
  email VARCHAR(65535),
  balance DECIMAL(18,2),
  tags SUPER, -- TODO(mog): collapsed from a Databricks nested type; element/field types were lost
  attrs SUPER, -- TODO(mog): collapsed from a Databricks nested type; element/field types were lost
  created_at TIMESTAMP,
  region VARCHAR(65535))
-- TODO(mog): dropped Databricks PARTITIONED BY (region) (set target partitioning / clustering manually)
-- TODO(mog): table comment dropped (use COMMENT ON in Redshift)
-- TODO(mog): dropped source LOCATION 's3://acme-lake/warehouse/sales.db/customers' (set target storage / external stage manually)

CREATE TABLE main.sales.customers (
  id BIGINT,
  name STRING,
  email STRING,
  balance DECIMAL(18,2),
  tags ARRAY<STRING>,
  attrs MAP<STRING, STRING>,
  created_at TIMESTAMP,
  region STRING)
USING delta
PARTITIONED BY (region)
COMMENT 'customer master table'
LOCATION 's3://acme-lake/warehouse/sales.db/customers'
TBLPROPERTIES (
  'delta.minReaderVersion' = '1',
  'delta.minWriterVersion' = '2',
  'delta.columnMapping.mode' = 'name',
  'delta.enableDeletionVectors' = 'true')

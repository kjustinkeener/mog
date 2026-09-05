CREATE TABLE my_dataset.customers
(
  id BIGINT NOT NULL,
  name VARCHAR,
  email VARCHAR COMMENT 'primary contact email',
  balance NUMBER(18, 2),
  attrs VARIANT,
  created_at TIMESTAMP_TZ,
  event_date DATE,
  region VARCHAR DEFAULT 'US'
)
-- TODO(mog): dropped BigQuery PARTITION BY (Snowflake auto-partitions via micro-partitions)
CLUSTER BY (region, name)
-- TODO(mog): dropped BigQuery table OPTIONS (labels / partition_expiration_days / require_partition_filter)
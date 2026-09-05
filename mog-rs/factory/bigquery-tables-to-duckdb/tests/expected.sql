CREATE TABLE customers
(
  id BIGINT NOT NULL,
  name VARCHAR,
  email VARCHAR,
  balance DECIMAL(18, 2),
  attrs JSON,
  created_at TIMESTAMPTZ,
  event_date DATE,
  region VARCHAR DEFAULT 'US'
)
-- TODO(mog): dropped BigQuery PARTITION BY (no DuckDB equivalent)
-- TODO(mog): dropped BigQuery CLUSTER BY (no DuckDB equivalent)
-- TODO(mog): dropped BigQuery table OPTIONS (labels / expiration / require_partition_filter)
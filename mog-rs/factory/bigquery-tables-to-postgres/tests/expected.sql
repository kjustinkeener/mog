CREATE TABLE customers
(
  id BIGINT NOT NULL,
  name TEXT,
  email TEXT,
  balance NUMERIC(18, 2),
  attrs JSONB,
  created_at TIMESTAMPTZ,
  event_date DATE,
  region TEXT DEFAULT 'US'
)
-- TODO(mog): dropped BigQuery PARTITION BY (use Postgres declarative partitioning if needed)
-- TODO(mog): dropped BigQuery CLUSTER BY (no direct Postgres table-DDL equivalent)
-- TODO(mog): dropped BigQuery table OPTIONS (labels / expiration / require_partition_filter)
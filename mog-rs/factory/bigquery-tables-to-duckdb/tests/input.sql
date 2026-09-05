CREATE TABLE `my-project.my_dataset.customers`
(
  id INT64 NOT NULL,
  name STRING,
  email STRING OPTIONS(description="primary contact email"),
  balance NUMERIC(18, 2),
  attrs JSON,
  created_at TIMESTAMP,
  event_date DATE,
  region STRING DEFAULT 'US'
)
PARTITION BY DATE(created_at)
CLUSTER BY region, name
OPTIONS(
  description="customer master table",
  labels=[("team", "data"), ("tier", "gold")],
  partition_expiration_days=90.0,
  require_partition_filter=TRUE
);

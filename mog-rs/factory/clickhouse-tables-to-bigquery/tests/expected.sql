CREATE OR REPLACE TABLE default.ch2bq_orders
(
    `order_id` INT64 NOT NULL,
    `customer_id` INT64 NOT NULL,
    `store_id` INT64 NOT NULL,
    `status_code` INT64 NOT NULL,
    `quantity` NUMERIC(20, 0) NOT NULL,
    `order_ref` STRING NOT NULL,
    `region_code` STRING NOT NULL,
    `currency` STRING NOT NULL,
    `total_amount` NUMERIC(18, 2) NOT NULL,
    `discount_rate` FLOAT64 NOT NULL,
    `weight_kg` FLOAT64 NOT NULL,
    `created_at` DATETIME NOT NULL,
    `updated_at` DATETIME NOT NULL,
    `order_date` DATE NOT NULL,
    `order_uuid` STRING NOT NULL,
    `notes` STRING,
    `priority` INT64 NOT NULL DEFAULT 5,
    `tags` ARRAY<STRING>,
    `attrs` JSON NOT NULL, -- TODO(mog): ClickHouse Map/Tuple/Nested flattened into BigQuery JSON; consider ARRAY<STRUCT<...>> if you need typed field access
    `status` STRING NOT NULL -- TODO(mog): ClickHouse enum mapped to STRING; the allowed-value set is not enforced in BigQuery
)
-- TODO(mog): dropped ClickHouse PARTITION BY toYYYYMM(order_date); BigQuery partitions on a DATE/TIMESTAMP column or an integer range, e.g. PARTITION BY DATE_TRUNC(order_date, MONTH)
-- TODO(mog): dropped ClickHouse PRIMARY KEY order_id (a sparse sort index, not an enforced constraint; BigQuery does not enforce keys)
-- TODO(mog): dropped ClickHouse ORDER BY (order_id, customer_id); consider BigQuery CLUSTER BY (order_id, customer_id) (max four columns)
-- TODO(mog): dropped ClickHouse SETTINGS index_granularity = 8192 (engine internals; no BigQuery equivalent)

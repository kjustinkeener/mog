CREATE TABLE default.ch2sf_orders
(
    `order_id` Int64,
    `customer_id` Int32,
    `store_id` Int16,
    `status_code` UInt8,
    `quantity` UInt64,
    `order_ref` String,
    `region_code` FixedString(4),
    `currency` LowCardinality(String),
    `total_amount` Decimal(18, 2),
    `discount_rate` Float32,
    `weight_kg` Float64,
    `created_at` DateTime64(3),
    `updated_at` DateTime,
    `order_date` Date,
    `order_uuid` UUID,
    `notes` Nullable(String),
    `priority` UInt8 DEFAULT 5,
    `tags` Array(String),
    `attrs` Map(String, String),
    `status` Enum8('new' = 1, 'shipped' = 2, 'done' = 3)
)
ENGINE = MergeTree
PARTITION BY toYYYYMM(order_date)
PRIMARY KEY order_id
ORDER BY (order_id, customer_id)
SETTINGS index_granularity = 8192

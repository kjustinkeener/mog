CREATE TABLE ch2pg_orders
(
    order_id UInt64,
    customer_id Int64,
    store_id Int32,
    region_code Int16,
    is_paid UInt8 DEFAULT 0,
    status LowCardinality(String),
    order_ref FixedString(12),
    notes Nullable(String),
    coupon LowCardinality(Nullable(String)),
    quantity UInt16,
    item_count UInt32,
    big_seq UInt64,
    amount Decimal(18, 2),
    discount Float32,
    tax_rate Float64,
    created_at DateTime64(3, 'UTC'),
    updated_at DateTime,
    order_date Date,
    row_uuid UUID,
    priority Enum8('low' = 1, 'med' = 2, 'high' = 3),
    tags Array(String),
    attrs Map(String, String)
);
-- TODO(mog): ClickHouse PARTITION BY not represented in Postgres table DDL; original PARTITION BY toYYYYMM(order_date)
-- TODO(mog): ClickHouse ORDER BY (sort key) has no Postgres table-DDL equivalent; original ORDER BY (order_id, customer_id)

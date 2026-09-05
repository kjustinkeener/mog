CREATE OR REPLACE TABLE ch2sf_orders
(
    order_id BIGINT NOT NULL,
    customer_id INTEGER NOT NULL,
    store_id SMALLINT NOT NULL,
    status_code SMALLINT NOT NULL,
    quantity NUMBER(20,0) NOT NULL,
    order_ref VARCHAR NOT NULL,
    region_code VARCHAR(4) NOT NULL,
    currency VARCHAR NOT NULL,
    total_amount NUMBER(18, 2) NOT NULL,
    discount_rate FLOAT NOT NULL,
    weight_kg FLOAT NOT NULL,
    created_at TIMESTAMP_NTZ NOT NULL,
    updated_at TIMESTAMP_NTZ NOT NULL,
    order_date DATE NOT NULL,
    order_uuid VARCHAR(36) NOT NULL,
    notes VARCHAR,
    priority SMALLINT DEFAULT 5 NOT NULL,
    tags ARRAY NOT NULL, -- TODO(mog): Snowflake ARRAY is untyped; the ClickHouse element type was dropped (verify ingestion/casting)
    attrs OBJECT NOT NULL, -- TODO(mog): Snowflake OBJECT is untyped; the ClickHouse Map key/value types were dropped
    status VARCHAR NOT NULL -- TODO(mog): ClickHouse enum mapped to VARCHAR; the allowed-value set is not enforced in Snowflake
)
-- TODO(mog): dropped ClickHouse PARTITION BY toYYYYMM(order_date) (Snowflake manages micro-partitions automatically)
-- TODO(mog): dropped ClickHouse PRIMARY KEY order_id (ClickHouse PK is a sparse sort index, not an enforced constraint)
-- TODO(mog): dropped ClickHouse ORDER BY (order_id, customer_id); Snowflake auto-clusters, but consider CLUSTER BY (order_id, customer_id) if this table is large
-- TODO(mog): dropped ClickHouse SETTINGS index_granularity = 8192 (engine internals; no Snowflake equivalent)

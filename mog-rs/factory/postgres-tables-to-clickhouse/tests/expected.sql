CREATE TABLE pg2ch_orders (
    id Int64,
    order_ref UUID DEFAULT generateUUIDv4(),
    customer_code String,
    description Nullable(String),
    quantity Int32,
    unit_price Decimal(12,2),
    discount Nullable(Float64),
    weight_kg Nullable(Float32),
    is_paid Bool DEFAULT false,
    status_code Nullable(Int16),
    created_at DateTime64(6, 'UTC') DEFAULT now(),
    updated_at Nullable(DateTime64(6)),
    order_date Nullable(Date),
    metadata Nullable(String),
    payload Nullable(String)
)
ENGINE = MergeTree
ORDER BY (id);

-- TODO(mog): dropped Postgres IDENTITY (sequence pg2ch_orders_id_seq); ClickHouse has no server-side auto-increment -- assign ids explicitly or via a DEFAULT

-- TODO(mog): dropped UNIQUE (customer_code); ClickHouse does not enforce uniqueness -- use ReplacingMergeTree or dedupe at query time

-- TODO(mog): PRIMARY KEY (id) consumed into ENGINE ORDER BY; ClickHouse has no enforced primary key

-- TODO(mog): dropped secondary index pg2ch_orders_created_at_idx on (created_at); add a ClickHouse data-skipping INDEX (TYPE minmax/set/bloom_filter) if needed


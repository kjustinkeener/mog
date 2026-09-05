CREATE TABLE ch2pg_orders
(
    order_id numeric(20, 0) NOT NULL,
    customer_id bigint NOT NULL,
    store_id integer NOT NULL,
    region_code smallint NOT NULL,
    is_paid smallint DEFAULT 0 NOT NULL,
    status text NOT NULL,
    order_ref varchar(12) NOT NULL,
    notes text,
    coupon text,
    quantity integer NOT NULL,
    item_count bigint NOT NULL,
    big_seq numeric(20, 0) NOT NULL,
    amount numeric(18, 2) NOT NULL,
    discount real NOT NULL,
    tax_rate double precision NOT NULL,
    created_at timestamptz NOT NULL,
    updated_at timestamp NOT NULL,
    order_date date NOT NULL,
    row_uuid uuid NOT NULL,
-- TODO(mog): ClickHouse Enum mapped to text; the allowed-value set was lost (use a CHECK constraint or a Postgres ENUM type)
    priority text NOT NULL,
-- TODO(mog): ClickHouse Array mapped to a Postgres array type; verify element type and nullability semantics
    tags text[] NOT NULL,
-- TODO(mog): ClickHouse Map mapped to jsonb; there is no native Postgres map/dictionary column type
    attrs jsonb NOT NULL
);
-- TODO(mog): ClickHouse PARTITION BY not represented in Postgres table DDL; original PARTITION BY toYYYYMM(order_date)
-- TODO(mog): ClickHouse ORDER BY (sort key) has no Postgres table-DDL equivalent; original ORDER BY (order_id, customer_id)



CREATE TABLE orders (
    id bigint NOT NULL,
    total DECIMAL(18,2),
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    note text,
    tags JSON -- TODO(mog): Postgres array mapped to JSON; MySQL has no array type
);

ALTER TABLE orders
    ADD CONSTRAINT orders_pkey PRIMARY KEY (id);


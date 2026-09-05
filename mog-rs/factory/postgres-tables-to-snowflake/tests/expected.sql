

CREATE TABLE orders (
    id bigint NOT NULL,
    total NUMBER(18,2),
    created_at TIMESTAMP_TZ DEFAULT CURRENT_TIMESTAMP(),
    note VARCHAR,
    tags ARRAY
);

ALTER TABLE orders
    ADD CONSTRAINT orders_pkey PRIMARY KEY (id);


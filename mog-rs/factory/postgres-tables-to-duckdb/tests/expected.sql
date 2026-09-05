

CREATE TABLE orders (
    id bigint NOT NULL,
    total numeric(18,2),
    created_at timestamp with time zone DEFAULT now(),
    note text,
    tags text[]
);

ALTER TABLE orders
    ADD CONSTRAINT orders_pkey PRIMARY KEY (id);


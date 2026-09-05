

CREATE TABLE orders (
    id bigint NOT NULL,
    total numeric(18,2),
    created_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP,
    note text,
    tags TEXT -- TODO(mog): Postgres array mapped to TEXT; SQLite has no array type
);

-- TODO(mog): SQLite cannot ALTER-add a PRIMARY KEY; fold it into the CREATE TABLE


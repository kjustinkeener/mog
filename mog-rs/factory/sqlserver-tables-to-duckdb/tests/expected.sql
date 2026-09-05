-- Database: lab

CREATE TABLE ms2dd_orders(
-- TODO(mog): SQL Server IDENTITY dropped; DuckDB has no auto-increment column. Create a SEQUENCE and set this column's DEFAULT to nextval('seq') to reproduce it.
	order_id bigint NOT NULL,
	order_uid UUID NOT NULL,
	customer_code VARCHAR(50) NOT NULL,
	description VARCHAR NULL,
	status_flag BOOLEAN NOT NULL,
	quantity int NOT NULL,
	unit_price DECIMAL(19,4) NOT NULL,
	discount_pct decimal(5, 2) NOT NULL,
	line_total AS (quantity*unit_price), -- TODO(mog): SQL Server stored computed column (PERSISTED) mapped to a DuckDB VIRTUAL generated column; it is recomputed on read, not materialized.
	notes VARCHAR NULL,
	payload BLOB NULL,
	created_at TIMESTAMP NOT NULL,
	updated_at TIMESTAMPTZ NULL,
	legacy_ts TIMESTAMP NULL
);

ALTER TABLE ms2dd_orders ADD CONSTRAINT PK_ms2dd_orders PRIMARY KEY
(
	order_id
);

CREATE UNIQUE INDEX UQ_ms2dd_code ON ms2dd_orders (customer_code);

CREATE INDEX IX_ms2dd_status ON ms2dd_orders
(
	status_flag
)
-- TODO(mog): SQL Server covering-index INCLUDE columns dropped; DuckDB indexes have no INCLUDE clause.
;

ALTER TABLE ms2dd_orders ALTER COLUMN order_uid SET DEFAULT uuid();

ALTER TABLE ms2dd_orders ALTER COLUMN status_flag SET DEFAULT (0);

ALTER TABLE ms2dd_orders ALTER COLUMN quantity SET DEFAULT (1);

ALTER TABLE ms2dd_orders ALTER COLUMN discount_pct SET DEFAULT (0.00);

ALTER TABLE ms2dd_orders ALTER COLUMN created_at SET DEFAULT now();

-- TODO(mog): DuckDB has no ALTER TABLE ADD CHECK; enforce this rule in your ETL/pipeline. Original: ALTER TABLE ms2dd_orders ADD CONSTRAINT CK_ms2dd_qty CHECK ((quantity>(0)));

COMMENT ON COLUMN ms2dd_orders.customer_code IS 'Business customer code';

COMMENT ON TABLE ms2dd_orders IS 'Customer orders fact table';


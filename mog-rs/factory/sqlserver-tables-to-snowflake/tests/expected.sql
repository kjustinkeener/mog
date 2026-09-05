-- Database: lab

CREATE OR REPLACE TABLE ms2sf_orders(
	order_id bigint IDENTITY(1,1) NOT NULL,
	order_uid VARCHAR(36) NOT NULL,
	customer_code VARCHAR(50) NOT NULL,
	description VARCHAR NULL,
	status_flag BOOLEAN NOT NULL,
	quantity int NOT NULL,
	unit_price NUMBER(19,4) NOT NULL,
	discount_pct NUMBER(5, 2) NOT NULL,
	-- TODO(mog): computed column line_total = (quantity*unit_price) [PERSISTED] has no Snowflake column form; recreate it as a view or a downstream derived column.
	notes VARCHAR NULL,
	payload BINARY NULL,
	created_at TIMESTAMP_NTZ(7) NOT NULL,
	updated_at TIMESTAMP_TZ(7) NULL,
	legacy_ts TIMESTAMP_NTZ NULL
);

ALTER TABLE ms2sf_orders ADD CONSTRAINT PK_ms2sf_orders PRIMARY KEY
(
	order_id
);

-- TODO(mog): Snowflake keys are metadata-only and ALTER ADD UNIQUE is not portable here; fold UNIQUE (customer_code) into the CREATE OR REPLACE TABLE for constraint UQ_ms2sf_code.

-- TODO(mog): Snowflake has no secondary indexes; CREATE INDEX IX_ms2sf_status was dropped (define a clustering key on the table if the access pattern needs it).

-- TODO(mog): set DEFAULT UUID_STRING() inline on column order_uid in the CREATE OR REPLACE TABLE (Snowflake ALTER COLUMN SET DEFAULT accepts only sequences).

-- TODO(mog): set DEFAULT (0) inline on column status_flag in the CREATE OR REPLACE TABLE (Snowflake ALTER COLUMN SET DEFAULT accepts only sequences).

-- TODO(mog): set DEFAULT (1) inline on column quantity in the CREATE OR REPLACE TABLE (Snowflake ALTER COLUMN SET DEFAULT accepts only sequences).

-- TODO(mog): set DEFAULT (0.00) inline on column discount_pct in the CREATE OR REPLACE TABLE (Snowflake ALTER COLUMN SET DEFAULT accepts only sequences).

-- TODO(mog): set DEFAULT CURRENT_TIMESTAMP() inline on column created_at in the CREATE OR REPLACE TABLE (Snowflake ALTER COLUMN SET DEFAULT accepts only sequences).

-- TODO(mog): Snowflake does not support CHECK constraints; enforce CK_ms2sf_qty ((quantity>(0))) in ELT or a view (dropped).

COMMENT ON COLUMN ms2sf_orders.customer_code IS 'Business customer code';

COMMENT ON TABLE ms2sf_orders IS 'Customer orders fact table';


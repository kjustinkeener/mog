-- Dataset: lab

CREATE OR REPLACE TABLE ms2bq_orders(
	order_id INT64 NOT NULL, -- TODO(mog): BigQuery has no auto-increment; assign this key with GENERATE_UUID() or from the application
	order_uid STRING NOT NULL,
	customer_code STRING(50) NOT NULL,
	description STRING NULL,
	status_flag BOOL NOT NULL,
	quantity INT64 NOT NULL,
	unit_price NUMERIC NOT NULL,
	discount_pct NUMERIC(5, 2) NOT NULL,
	-- TODO(mog): computed column line_total = (quantity*unit_price) [PERSISTED] has no BigQuery column form; recreate it as a view or a downstream derived column.
	notes STRING NULL,
	payload BYTES NULL,
	created_at DATETIME NOT NULL,
	updated_at TIMESTAMP NULL,
	legacy_ts DATETIME NULL
);

-- TODO(mog): BigQuery takes PRIMARY KEY (order_id) NOT ENFORCED only inside CREATE TABLE; fold constraint PK_ms2bq_orders into the table (dropped).

-- TODO(mog): BigQuery has no UNIQUE constraint; enforce uniqueness of (customer_code) in ELT or a check query (constraint UQ_ms2bq_code dropped).

-- TODO(mog): BigQuery has no secondary indexes; CREATE INDEX IX_ms2bq_status was dropped (use partitioning or clustering on the table instead).

-- TODO(mog): fold DEFAULT GENERATE_UUID() inline onto column order_uid in the table definition (BigQuery has no ALTER ADD DEFAULT-constraint form).

-- TODO(mog): fold DEFAULT (0) inline onto column status_flag in the table definition (BigQuery has no ALTER ADD DEFAULT-constraint form).

-- TODO(mog): fold DEFAULT (1) inline onto column quantity in the table definition (BigQuery has no ALTER ADD DEFAULT-constraint form).

-- TODO(mog): fold DEFAULT (0.00) inline onto column discount_pct in the table definition (BigQuery has no ALTER ADD DEFAULT-constraint form).

-- TODO(mog): fold DEFAULT CURRENT_DATETIME() inline onto column created_at in the table definition (BigQuery has no ALTER ADD DEFAULT-constraint form).

-- TODO(mog): BigQuery does not support CHECK constraints; enforce CK_ms2bq_qty ((quantity>(0))) in ELT or a view (dropped).


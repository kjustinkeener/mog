
CREATE TABLE IF NOT EXISTS customers
(
	id BIGINT
	,name VARCHAR(512)
	,balance NUMERIC(18,2)
	,created_at TIMESTAMP
	,region VARCHAR(64)
)
-- TODO(mog): dropped Redshift DISTSTYLE (no DuckDB equivalent)
-- TODO(mog): dropped Redshift DISTKEY
-- TODO(mog): dropped Redshift SORTKEY
;


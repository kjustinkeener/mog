
CREATE TABLE IF NOT EXISTS customers
(
	id BIGINT
	,name VARCHAR(512)
	,balance NUMERIC(18,2)
	,created_at TIMESTAMP_NTZ
	,region VARCHAR(64)
)
-- TODO(mog): dropped Redshift DISTSTYLE (Snowflake has no distribution style)
-- TODO(mog): dropped Redshift DISTKEY
-- TODO(mog): dropped Redshift SORTKEY (consider Snowflake CLUSTER BY)
;


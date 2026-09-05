
CREATE TABLE IF NOT EXISTS customers
(
	id BIGINT
	,name STRING
	,balance DECIMAL(18,2)
	,created_at TIMESTAMP
	,region STRING
)
-- TODO(mog): dropped Redshift DISTSTYLE
-- TODO(mog): dropped Redshift DISTKEY
-- TODO(mog): dropped Redshift SORTKEY
;


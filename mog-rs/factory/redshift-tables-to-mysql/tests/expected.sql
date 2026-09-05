
CREATE TABLE IF NOT EXISTS customers
(
	id BIGINT
	,name VARCHAR(512)
	,balance DECIMAL(18,2)
	,created_at DATETIME
	,region VARCHAR(64)
)
-- TODO(mog): dropped Redshift DISTSTYLE
-- TODO(mog): dropped Redshift DISTKEY
-- TODO(mog): dropped Redshift SORTKEY
;


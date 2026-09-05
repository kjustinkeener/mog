
CREATE TABLE IF NOT EXISTS "public"."customers"
(
	"id" BIGINT
	,"name" VARCHAR(512)
	,"balance" NUMERIC(18,2)
	,"created_at" TIMESTAMP WITHOUT TIME ZONE
	,"region" VARCHAR(64)
)
-- TODO(mog): dropped Redshift DISTSTYLE (no Postgres equivalent)
-- TODO(mog): dropped Redshift DISTKEY
-- TODO(mog): dropped Redshift SORTKEY
;


--DROP TABLE "public"."customers";
CREATE TABLE IF NOT EXISTS "public"."customers"
(
	"id" BIGINT ENCODE az64
	,"name" VARCHAR(512) ENCODE zstd
	,"balance" NUMERIC(18,2) ENCODE az64
	,"created_at" TIMESTAMP WITHOUT TIME ZONE ENCODE az64
	,"region" VARCHAR(64) ENCODE bytedict
)
DISTSTYLE KEY
 DISTKEY ("id")
 SORTKEY (
	"region"
	)
;
ALTER TABLE "public"."customers" owner to "admin";

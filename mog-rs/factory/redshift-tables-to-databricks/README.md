# Redshift DDL to Databricks

Convert a Redshift v_generate_tbl_ddl export to Databricks/Spark DDL

Convert Redshift v_generate_tbl_ddl output to Databricks/Spark SQL DDL: strips the Redshift plumbing and physical clauses (DISTSTYLE/DISTKEY/SORTKEY/ENCODE, flagged), unquotes identifiers, and maps types (VARCHAR(n)/CHAR(n)->STRING, NUMERIC->DECIMAL, TIMESTAMP WITHOUT/WITH TIME ZONE->TIMESTAMP, SUPER->STRING). Dropped physical clauses are flagged inline as TODO(mog).

## Run

```
mog -m redshift-tables-to-databricks <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
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
```

Output:

```

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
```

## Steps

- `eol_lf`
- `replace_regex`
- `replace_regex`
- `replace_regex`
- `replace_regex`
- `replace_regex`
- `replace_regex`
- `replace_regex`
- `replace_regex`
- `replace_regex`
- `replace_regex`
- `replace`
- `replace`
- `replace_regex`

## Tags

`sql` `redshift` `databricks` `spark` `convert` `migration`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

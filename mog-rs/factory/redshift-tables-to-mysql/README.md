# Redshift DDL to MySQL

Convert a Redshift v_generate_tbl_ddl export to MySQL DDL

Convert Redshift v_generate_tbl_ddl output to MySQL DDL: strips the --DROP/--WARNING comments, owner-to trailer, ENCODE, and DISTSTYLE/DISTKEY/SORTKEY (flagged), unquotes identifiers, and maps types (NUMERIC->DECIMAL, TIMESTAMP WITHOUT TIME ZONE->DATETIME, TIMESTAMP WITH TIME ZONE->TIMESTAMP, SUPER->JSON). Dropped physical clauses are flagged inline as TODO(mog).

## Run

```
mog -m redshift-tables-to-mysql <file>
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
	,name VARCHAR(512)
	,balance DECIMAL(18,2)
	,created_at DATETIME
	,region VARCHAR(64)
)
-- TODO(mog): dropped Redshift DISTSTYLE
-- TODO(mog): dropped Redshift DISTKEY
-- TODO(mog): dropped Redshift SORTKEY
;
```

## Pipeline

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

`sql` `redshift` `mysql` `convert` `migration`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

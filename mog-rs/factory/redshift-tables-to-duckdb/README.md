# Redshift DDL to DuckDB

Convert a Redshift v_generate_tbl_ddl export to DuckDB DDL

Convert Redshift v_generate_tbl_ddl (or SHOW TABLE) output to DuckDB DDL: strips the --DROP/--WARNING comments and the ALTER TABLE ... owner to trailer, removes ENCODE compression and the DISTSTYLE/DISTKEY/SORTKEY physical clauses (flagged), unquotes identifiers and drops the public. qualifier, and maps TIMESTAMP WITHOUT/WITH TIME ZONE to TIMESTAMP/TIMESTAMPTZ. BIGINT/VARCHAR/NUMERIC/CHAR pass through to DuckDB as-is. Dropped physical clauses are flagged inline as TODO(mog). Validated by executing the result in a real DuckDB engine.

## Run

```
mog -m redshift-tables-to-duckdb <file>
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
	,balance NUMERIC(18,2)
	,created_at TIMESTAMP
	,region VARCHAR(64)
)
-- TODO(mog): dropped Redshift DISTSTYLE (no DuckDB equivalent)
-- TODO(mog): dropped Redshift DISTKEY
-- TODO(mog): dropped Redshift SORTKEY
;
```

## Pipeline

- `eol_lf`: Normalize to LF.
- `replace_regex`: Drop --DROP / --WARNING comment lines.
- `replace_regex`: Drop the ALTER TABLE ... owner to trailer.
- `replace_regex`: Remove per-column ENCODE compression.
- `replace_regex`: Drop DISTSTYLE (flagged).
- `replace_regex`: Drop DISTKEY (flagged).
- `replace_regex`: Drop the (multi-line) SORTKEY (flagged).
- `replace_regex`: TIMESTAMP WITHOUT TIME ZONE -> TIMESTAMP.
- `replace_regex`: TIMESTAMP WITH TIME ZONE -> TIMESTAMPTZ.
- `replace`: Drop the public. schema qualifier.
- `replace`: Unquote the everything-quoted Redshift identifiers.
- `replace_regex`: Collapse blank lines left behind.

## Tags

`sql` `redshift` `duckdb` `warehouse` `convert` `migration`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

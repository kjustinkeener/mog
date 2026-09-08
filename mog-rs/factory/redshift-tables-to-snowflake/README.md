# Redshift DDL to Snowflake

Convert a Redshift v_generate_tbl_ddl export to Snowflake DDL

Convert Redshift v_generate_tbl_ddl (or SHOW TABLE) output to Snowflake DDL: strips the --DROP/--WARNING comment lines and the ALTER TABLE ... owner to trailer, removes ENCODE compression and the DISTSTYLE/DISTKEY/SORTKEY physical clauses (flagged, since Snowflake has no distribution/sort keys), unquotes identifiers and drops the public. schema qualifier, and maps TIMESTAMP WITHOUT/WITH TIME ZONE to TIMESTAMP_NTZ/TIMESTAMP_TZ. Not a semantic transpiler; dropped physical clauses are flagged inline as TODO(mog). Validated by executing the result in a real Snowflake engine (fakesnow).

## Run

```
mog -m redshift-tables-to-snowflake <file>
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
	,created_at TIMESTAMP_NTZ
	,region VARCHAR(64)
)
-- TODO(mog): dropped Redshift DISTSTYLE (Snowflake has no distribution style)
-- TODO(mog): dropped Redshift DISTKEY
-- TODO(mog): dropped Redshift SORTKEY (consider Snowflake CLUSTER BY)
;
```

## Steps

- `eol_lf`: Normalize to LF.
- `replace_regex`: Drop --DROP / --WARNING comment lines.
- `replace_regex`: Drop the ALTER TABLE ... owner to trailer.
- `replace_regex`: Remove per-column ENCODE compression (Snowflake auto-compresses).
- `replace_regex`: Drop DISTSTYLE (flagged).
- `replace_regex`: Drop DISTKEY (flagged).
- `replace_regex`: Drop the (multi-line) SORTKEY (flagged; consider CLUSTER BY).
- `replace_regex`: TIMESTAMP WITHOUT TIME ZONE -> TIMESTAMP_NTZ.
- `replace_regex`: TIMESTAMP WITH TIME ZONE -> TIMESTAMP_TZ.
- `replace`: Drop the public. schema qualifier.
- `replace`: Unquote the everything-quoted Redshift identifiers.
- `replace_regex`: Collapse blank lines left behind.

## Tags

`sql` `redshift` `snowflake` `warehouse` `convert` `migration`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

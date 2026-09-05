# Redshift DDL to Postgres

Convert a Redshift v_generate_tbl_ddl export to PostgreSQL DDL

Convert Redshift v_generate_tbl_ddl (or SHOW TABLE) output to PostgreSQL DDL. Redshift SQL is Postgres-derived, so the work is mostly removal: strips the --DROP/--WARNING comments and the ALTER TABLE ... owner to trailer, and removes ENCODE compression and the DISTSTYLE/DISTKEY/SORTKEY physical clauses (flagged), which have no Postgres equivalent. Types (BIGINT, VARCHAR, NUMERIC, CHAR, TIMESTAMP WITHOUT/WITH TIME ZONE) and quoted identifiers pass straight through. Dropped physical clauses are flagged inline as TODO(mog). Validated by executing the result in a real PostgreSQL engine.

## Run

```
mog -m redshift-tables-to-postgres <file>
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
```

## Pipeline

- `eol_lf`: Normalize to LF.
- `replace_regex`: Drop --DROP / --WARNING comment lines.
- `replace_regex`: Drop the ALTER TABLE ... owner to trailer.
- `replace_regex`: Remove per-column ENCODE compression.
- `replace_regex`: Drop DISTSTYLE (flagged).
- `replace_regex`: Drop DISTKEY (flagged).
- `replace_regex`: Drop the (multi-line) SORTKEY (flagged).
- `replace_regex`: Collapse blank lines left behind.

## Tags

`sql` `redshift` `postgres` `convert` `migration`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

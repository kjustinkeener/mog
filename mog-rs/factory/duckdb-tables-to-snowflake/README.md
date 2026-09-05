# DuckDB to Snowflake

Convert a DuckDB table definition to Snowflake DDL

Convert DuckDB DDL (e.g. duckdb_tables().sql) to Snowflake: maps JSON->VARIANT, TIMESTAMP->TIMESTAMP_NTZ, TIMESTAMPTZ->TIMESTAMP_TZ, VARCHAR[]/list->ARRAY (flagged), and rewrites CREATE TABLE to CREATE OR REPLACE TABLE. Emits TODO(mog) markers where a construct cannot be mapped mechanically.

## Run

```
mog -m duckdb-tables-to-snowflake <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
CREATE TABLE events(
  id BIGINT,
  name VARCHAR,
  payload JSON,
  amount DECIMAL(18,2),
  created_at TIMESTAMP,
  updated_at TIMESTAMPTZ,
  tags VARCHAR[]
);
```

Output:

```
CREATE OR REPLACE TABLE events(
  id BIGINT,
  name VARCHAR,
  payload VARIANT,
  amount DECIMAL(18,2),
  created_at TIMESTAMP_NTZ,
  updated_at TIMESTAMP_TZ,
  tags ARRAY -- TODO(mog): DuckDB LIST/array mapped to Snowflake ARRAY (untyped); element type lost
);
```

## Pipeline

- `eol_lf`
- `replace_regex`
- `flag_matching`
- `replace_regex`
- `replace_regex`
- `replace_regex`
- `replace_regex`

## Tags

`sql` `duckdb` `snowflake` `convert` `migration`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

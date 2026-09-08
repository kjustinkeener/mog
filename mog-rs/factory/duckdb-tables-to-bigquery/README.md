# DuckDB to BigQuery (DDL/SQL)

Convert DuckDB SQL to BigQuery

Rewrite DuckDB SQL toward BigQuery: :: casts to CAST(...), double-quoted identifiers to backticks, and type keywords (VARCHAR/TEXT/CHAR->STRING, BIGINT/INT/SMALLINT/HUGEINT->INT64, DOUBLE/REAL->FLOAT64, DECIMAL->NUMERIC, BOOLEAN->BOOL, BLOB->BYTES, TIMESTAMPTZ->TIMESTAMP, TIMESTAMP->DATETIME, UUID->STRING). The inverse of bigquery-tables-to-duckdb. Minimal-diff and regex-based, not a semantic transpiler. Assumes LF text.

## Run

```
mog -m duckdb-tables-to-bigquery <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
CREATE TABLE "users" (
  id BIGINT,
  name VARCHAR,
  score DOUBLE,
  active BOOLEAN,
  created TIMESTAMPTZ,
  local_ts TIMESTAMP,
  raw BLOB
);
SELECT id::VARCHAR FROM "users";
```

Output:

```
CREATE TABLE `users` (
  id INT64,
  name STRING,
  score FLOAT64,
  active BOOL,
  created TIMESTAMP,
  local_ts DATETIME,
  raw BYTES
);
SELECT CAST(id AS STRING) FROM `users`;
```

## Steps

- `run_mog`: Rewrite expr::TYPE to CAST(expr AS TYPE) (BigQuery has no ::). Run before requoting.
- `run_mog`: DuckDB "id" identifiers to BigQuery `id` backticks.
- `replace_map`: Map DuckDB type keywords to BigQuery (whole-word, case-insensitive; longest match wins).

## Tags

`sql` `duckdb` `bigquery` `warehouse` `convert` `migration`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

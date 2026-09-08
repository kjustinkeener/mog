# PostgreSQL to BigQuery (DDL/SQL)

Convert PostgreSQL SQL to BigQuery

Rewrite PostgreSQL SQL toward BigQuery: :: casts to CAST(...), double-quoted identifiers to backticks, SERIAL family to INT64 (BigQuery has no auto-increment), and type keywords (TEXT/VARCHAR->STRING, BOOLEAN->BOOL, BYTEA->BYTES, TIMESTAMPTZ->TIMESTAMP, TIMESTAMP->DATETIME, DOUBLE PRECISION/REAL->FLOAT64, JSONB/JSON->JSON, UUID->STRING). Minimal-diff regex rewrite, NOT a semantic transpiler. Assumes LF text.

## Run

```
mog -m postgres-tables-to-bigquery <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
CREATE TABLE "users" (
  id SERIAL,
  name TEXT,
  active BOOLEAN,
  data JSONB,
  created TIMESTAMPTZ,
  local_ts TIMESTAMP
);
SELECT id::TEXT FROM "users";
```

Output:

```
CREATE TABLE `users` (
  id INT64,
  name STRING,
  active BOOL,
  data JSON,
  created TIMESTAMP,
  local_ts DATETIME
);
SELECT CAST(id AS STRING) FROM `users`;
```

## Steps

- `run_mog`: :: -> CAST()
- `run_mog`: "id" -> `id`
- `replace_map`

## Tags

`sql` `postgres` `bigquery` `warehouse` `convert` `migration`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

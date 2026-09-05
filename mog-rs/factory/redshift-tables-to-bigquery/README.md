# Redshift to BigQuery (DDL/SQL)

Convert Amazon Redshift SQL to BigQuery

Convert Amazon Redshift SQL to BigQuery: rewrites :: casts to CAST(...), double-quoted identifiers to backticks, and type keywords (VARCHAR/CHAR->STRING, INTEGER/INT/SMALLINT/BIGINT->INT64, REAL->FLOAT64, DOUBLE PRECISION->FLOAT64, DECIMAL/NUMERIC->NUMERIC, BOOLEAN->BOOL, TIMESTAMPTZ->TIMESTAMP, TIMESTAMP->DATETIME, SUPER->JSON). Not a semantic transpiler (no DISTKEY/SORTKEY, no GETDATE/window rewrites). Assumes LF text.

## Run

```
mog -m redshift-tables-to-bigquery <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
CREATE TABLE "events" (
  id BIGINT,
  name VARCHAR,
  amount DECIMAL,
  created TIMESTAMPTZ,
  local_ts TIMESTAMP,
  payload SUPER
);
SELECT id::VARCHAR FROM "events";
```

Output:

```
CREATE TABLE `events` (
  id INT64,
  name STRING,
  amount NUMERIC,
  created TIMESTAMP,
  local_ts DATETIME,
  payload JSON
);
SELECT CAST(id AS STRING) FROM `events`;
```

## Pipeline

- `run_mog`: :: -> CAST()
- `run_mog`: "id" -> `id`
- `replace_map`

## Tags

`sql` `redshift` `bigquery` `warehouse` `convert` `migration`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

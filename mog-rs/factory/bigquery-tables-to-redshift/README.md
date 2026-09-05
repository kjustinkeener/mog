# BigQuery to Redshift (DDL/SQL)

Convert BigQuery SQL to Amazon Redshift

Convert BigQuery SQL toward Amazon Redshift: backtick-quoted identifiers to double quotes, and type keywords (STRING->VARCHAR, INT64->BIGINT, FLOAT64->DOUBLE PRECISION, NUMERIC->DECIMAL, BOOL->BOOLEAN, BYTES->VARBYTE, TIMESTAMP->TIMESTAMPTZ, DATETIME->TIMESTAMP, JSON->SUPER). Redshift accepts CAST and ::, so casts pass through. The inverse of redshift-tables-to-bigquery. Minimal-diff, not a semantic transpiler. Assumes LF text.

## Run

```
mog -m bigquery-tables-to-redshift <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
CREATE TABLE `events` (
  id INT64,
  name STRING,
  amount NUMERIC,
  created TIMESTAMP,
  local_ts DATETIME,
  payload JSON
);
```

Output:

```
CREATE TABLE "events" (
  id BIGINT,
  name VARCHAR,
  amount DECIMAL,
  created TIMESTAMPTZ,
  local_ts TIMESTAMP,
  payload SUPER
);
```

## Pipeline

- `run_mog`: `id` -> "id"
- `replace_map`

## Tags

`sql` `bigquery` `redshift` `warehouse` `convert` `migration`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

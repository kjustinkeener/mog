# Databricks (Spark SQL) to BigQuery (DDL/SQL)

Convert Databricks Spark SQL to BigQuery

Convert Databricks / Spark SQL toward BigQuery. Both quote identifiers with backticks, so those pass through; complex types (ARRAY<...>, MAP<...>, STRUCT<...>) are collapsed to JSON, the USING <format> table clause (e.g. USING DELTA) is dropped, and scalar type keywords are mapped (LONG/INT/INTEGER/BIGINT/SMALLINT/SHORT/TINYINT/BYTE->INT64, DOUBLE/FLOAT/REAL->FLOAT64, DECIMAL->NUMERIC, BOOLEAN->BOOL, BINARY->BYTES; STRING/TIMESTAMP/DATE stay) with NVL->IFNULL. Minimal-diff, not a semantic transpiler: nested complex types, STRUCT/ARRAY semantics, LATERAL VIEW/EXPLODE, and Spark-only functions are left for manual review. Assumes LF text.

## Run

```
mog -m databricks-tables-to-bigquery <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
CREATE TABLE `main`.`events` (
  `event_id` LONG,
  `count` INT,
  `code` SHORT,
  `flag` BYTE,
  `amount` DECIMAL(10,2),
  `ratio` DOUBLE,
  `name` STRING,
  `tags` ARRAY<STRING>,
  `attrs` MAP<STRING, STRING>,
  `raw` BINARY,
  `is_active` BOOLEAN,
  `created_at` TIMESTAMP
)
USING DELTA;

SELECT
  `event_id`,
```

_(... 2 more line(s))_

Output:

```
CREATE TABLE `main`.`events` (
  `event_id` INT64,
  `count` INT64,
  `code` INT64,
  `flag` INT64,
  `amount` NUMERIC(10,2),
  `ratio` FLOAT64,
  `name` STRING,
  `tags` JSON,
  `attrs` JSON,
  `raw` BYTES,
  `is_active` BOOL,
  `created_at` TIMESTAMP
);

SELECT
  `event_id`,
  IFNULL(`name`, 'x') AS name
```

_(... 1 more line(s))_

## Pipeline

- `replace_regex`: Collapse ARRAY<...>/MAP<...>/STRUCT<...> to JSON (best-effort; non-nested).
- `replace_regex`: Drop the USING <format> table clause (e.g. USING DELTA).
- `replace_map`: Map Spark scalar type keywords to BigQuery types (whole-word, case-insensitive; longest match wins).
- `replace_map`: Safe 1:1 function rename (same argument shape).

## Tags

`sql` `databricks` `spark` `bigquery` `warehouse` `convert` `migration`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

# BigQuery DDL to Snowflake

Convert a BigQuery INFORMATION_SCHEMA.ddl export to Snowflake DDL

Convert BigQuery's INFORMATION_SCHEMA.TABLES.ddl (console 'Copy DDL') output to Snowflake. Drops the backtick project.dataset.table quoting and project prefix, rewrites per-column OPTIONS(description=...) to Snowflake COMMENT, drops the table-level OPTIONS(...) block and PARTITION BY (flagged), rewrites CLUSTER BY a,b to Snowflake CLUSTER BY (a,b), and maps GoogleSQL types to Snowflake (INT64->BIGINT, FLOAT64->FLOAT, STRING->VARCHAR, BYTES->BINARY, BOOL->BOOLEAN, NUMERIC->NUMBER, JSON->VARIANT, DATETIME->TIMESTAMP_NTZ, TIMESTAMP->TIMESTAMP_TZ).

## Run

```
mog -m bigquery-tables-to-snowflake <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
CREATE TABLE `my-project.my_dataset.customers`
(
  id INT64 NOT NULL,
  name STRING,
  email STRING OPTIONS(description="primary contact email"),
  balance NUMERIC(18, 2),
  attrs JSON,
  created_at TIMESTAMP,
  event_date DATE,
  region STRING DEFAULT 'US'
)
PARTITION BY DATE(created_at)
CLUSTER BY region, name
OPTIONS(
  description="customer master table",
  labels=[("team", "data"), ("tier", "gold")],
  partition_expiration_days=90.0,
  require_partition_filter=TRUE
```

_(... 1 more line(s))_

Output:

```
CREATE TABLE my_dataset.customers
(
  id BIGINT NOT NULL,
  name VARCHAR,
  email VARCHAR COMMENT 'primary contact email',
  balance NUMBER(18, 2),
  attrs VARIANT,
  created_at TIMESTAMP_TZ,
  event_date DATE,
  region VARCHAR DEFAULT 'US'
)
-- TODO(mog): dropped BigQuery PARTITION BY (Snowflake auto-partitions via micro-partitions)
CLUSTER BY (region, name)
-- TODO(mog): dropped BigQuery table OPTIONS (labels / partition_expiration_days / require_partition_filter)
```

## Pipeline

- `eol_lf`: Normalize to LF.
- `replace_regex`: Drop the trailing table OPTIONS(...) block (labels/expiration/etc).
- `replace_regex`: Drop PARTITION BY (Snowflake auto-partitions).
- `replace_regex`: BigQuery CLUSTER BY a, b -> Snowflake CLUSTER BY (a, b).
- `replace_regex`: Drop the `project.dataset.table` backticks and the project prefix.
- `replace_regex`: Per-column OPTIONS(description=...) -> Snowflake COMMENT.
- `replace_regex`: NUMERIC / BIGNUMERIC -> NUMBER (keeps the (p,s)).
- `replace_regex`: DATETIME -> TIMESTAMP_NTZ (wall-clock).
- `replace_regex`: TIMESTAMP (instant) -> TIMESTAMP_TZ.
- `replace_map`: Map remaining GoogleSQL scalar types to Snowflake.

## Tags

`sql` `bigquery` `snowflake` `warehouse` `convert` `migration`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

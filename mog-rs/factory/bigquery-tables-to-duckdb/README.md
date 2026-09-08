# BigQuery DDL to DuckDB

Convert a BigQuery INFORMATION_SCHEMA.ddl export to DuckDB DDL

Convert BigQuery's INFORMATION_SCHEMA.TABLES.ddl output to DuckDB. Reduces the backtick project.dataset.table name to the bare table, drops per-column OPTIONS(description=...), the table OPTIONS(...) block, PARTITION BY, and CLUSTER BY (all flagged; DuckDB has no clustering/partitioning DDL), and maps GoogleSQL types to DuckDB (INT64->BIGINT, FLOAT64->DOUBLE, STRING->VARCHAR, BYTES->BLOB, BOOL->BOOLEAN, NUMERIC->DECIMAL, DATETIME->TIMESTAMP, TIMESTAMP->TIMESTAMPTZ; JSON stays JSON).

## Run

```
mog -m bigquery-tables-to-duckdb <file>
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
CREATE TABLE customers
(
  id BIGINT NOT NULL,
  name VARCHAR,
  email VARCHAR,
  balance DECIMAL(18, 2),
  attrs JSON,
  created_at TIMESTAMPTZ,
  event_date DATE,
  region VARCHAR DEFAULT 'US'
)
-- TODO(mog): dropped BigQuery PARTITION BY (no DuckDB equivalent)
-- TODO(mog): dropped BigQuery CLUSTER BY (no DuckDB equivalent)
-- TODO(mog): dropped BigQuery table OPTIONS (labels / expiration / require_partition_filter)
```

## Steps

- `eol_lf`: Normalize to LF.
- `replace_regex`: Drop the trailing table OPTIONS(...) block.
- `replace_regex`: Drop PARTITION BY (DuckDB has no partitioning DDL).
- `replace_regex`: Drop CLUSTER BY (DuckDB has no clustering DDL).
- `replace_regex`: Drop per-column OPTIONS(description=...) (DuckDB has no inline column comment).
- `replace_regex`: Reduce `project.dataset.table` to the bare table name.
- `replace_regex`: NUMERIC / BIGNUMERIC -> DECIMAL (keeps the (p,s)).
- `replace_regex`: DATETIME -> TIMESTAMP.
- `replace_regex`: TIMESTAMP (instant) -> TIMESTAMPTZ.
- `replace_map`: Map remaining GoogleSQL scalar types to DuckDB.

## Tags

`sql` `bigquery` `duckdb` `warehouse` `convert` `migration`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

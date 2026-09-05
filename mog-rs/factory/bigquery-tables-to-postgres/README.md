# BigQuery DDL to Postgres

Convert a BigQuery INFORMATION_SCHEMA.ddl export to PostgreSQL DDL

Convert BigQuery's INFORMATION_SCHEMA.TABLES.ddl output to PostgreSQL. Reduces the backtick project.dataset.table name to the bare table, drops per-column OPTIONS(description=...) and the table OPTIONS(...)/PARTITION BY/CLUSTER BY clauses (all flagged), and maps GoogleSQL types to Postgres (INT64->BIGINT, FLOAT64->DOUBLE PRECISION, STRING->TEXT, BYTES->BYTEA, BOOL->BOOLEAN, NUMERIC->NUMERIC, JSON->JSONB, DATETIME->TIMESTAMP, TIMESTAMP->TIMESTAMPTZ).

## Run

```
mog -m bigquery-tables-to-postgres <file>
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
  name TEXT,
  email TEXT,
  balance NUMERIC(18, 2),
  attrs JSONB,
  created_at TIMESTAMPTZ,
  event_date DATE,
  region TEXT DEFAULT 'US'
)
-- TODO(mog): dropped BigQuery PARTITION BY (use Postgres declarative partitioning if needed)
-- TODO(mog): dropped BigQuery CLUSTER BY (no direct Postgres table-DDL equivalent)
-- TODO(mog): dropped BigQuery table OPTIONS (labels / expiration / require_partition_filter)
```

## Pipeline

- `eol_lf`: Normalize to LF.
- `replace_regex`: Drop the trailing table OPTIONS(...) block.
- `replace_regex`: Drop PARTITION BY (Postgres partitioning is declared differently).
- `replace_regex`: Drop CLUSTER BY (Postgres CLUSTER is a maintenance command, not table DDL).
- `replace_regex`: Drop per-column OPTIONS(description=...) (Postgres uses COMMENT ON).
- `replace_regex`: Reduce `project.dataset.table` to the bare table name.
- `replace_regex`: NUMERIC / BIGNUMERIC -> NUMERIC (keeps the (p,s)).
- `replace_regex`: TIMESTAMP (instant) -> TIMESTAMPTZ. Runs before DATETIME so it does not re-match.
- `replace_regex`: DATETIME (wall-clock) -> TIMESTAMP.
- `replace_map`: Map remaining GoogleSQL scalar types to Postgres.

## Tags

`sql` `bigquery` `postgres` `warehouse` `convert` `migration`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

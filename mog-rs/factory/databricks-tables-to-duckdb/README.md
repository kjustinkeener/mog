# Databricks SHOW CREATE TABLE to DuckDB

Convert a Databricks/Spark table export to DuckDB DDL

Convert Databricks 'SHOW CREATE TABLE' output to DuckDB. Strips Delta/Spark export noise (USING/LOCATION/TBLPROPERTIES, PARTITIONED BY/CLUSTER BY flagged), maps Spark scalar types to DuckDB (STRING->VARCHAR, LONG->BIGINT, SHORT->SMALLINT, BYTE->TINYINT), collapses nested ARRAY/MAP/STRUCT<...> to JSON (flagged), maps TIMESTAMP (instant) to TIMESTAMPTZ, and drops the trailing table COMMENT (DuckDB has no inline table comment).

## Run

```
mog -m databricks-tables-to-duckdb <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
CREATE TABLE main.sales.customers (
  id BIGINT,
  name STRING,
  email STRING,
  balance DECIMAL(18,2),
  tags ARRAY<STRING>,
  attrs MAP<STRING, STRING>,
  created_at TIMESTAMP,
  region STRING)
USING delta
PARTITIONED BY (region)
COMMENT 'customer master table'
LOCATION 's3://acme-lake/warehouse/sales.db/customers'
TBLPROPERTIES (
  'delta.minReaderVersion' = '1',
  'delta.minWriterVersion' = '2',
  'delta.columnMapping.mode' = 'name',
  'delta.enableDeletionVectors' = 'true')
```

Output:

```
CREATE TABLE customers (
  id BIGINT,
  name VARCHAR,
  email VARCHAR,
  balance DECIMAL(18,2),
  tags JSON, -- TODO(mog): collapsed from a Databricks nested type; element/field types were lost
  attrs JSON, -- TODO(mog): collapsed from a Databricks nested type; element/field types were lost
  created_at TIMESTAMPTZ,
  region VARCHAR)
-- TODO(mog): dropped Databricks PARTITIONED BY (region) (set target partitioning / clustering manually)
-- TODO(mog): table comment dropped (use COMMENT ON TABLE in DuckDB)
-- TODO(mog): dropped source LOCATION 's3://acme-lake/warehouse/sales.db/customers' (set target storage / external stage manually)
```

## Pipeline

- `run_mog`
- `replace_regex`: Reduce catalog.schema.table to the bare table (DuckDB has no Unity catalog).
- `replace_regex`: Collapse nested ARRAY/MAP/STRUCT<...> to JSON (flagged separately).
- `flag_matching`: Flag JSON columns collapsed from a Databricks nested type.
- `replace_regex`: Spark TIMESTAMP (instant) -> DuckDB TIMESTAMPTZ.
- `replace_map`: Map Spark scalar type keywords to DuckDB.
- `replace_regex`: Drop the trailing table COMMENT (DuckDB has no inline table comment).

## Tags

`sql` `databricks` `spark` `duckdb` `warehouse` `convert` `migration`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

# Databricks SHOW CREATE TABLE to Postgres

Convert a Databricks/Spark table export to PostgreSQL DDL

Convert Databricks 'SHOW CREATE TABLE' output to PostgreSQL. Strips Delta/Spark export noise (USING/LOCATION/TBLPROPERTIES, PARTITIONED BY/CLUSTER BY flagged), reduces catalog.schema.table to schema.table, maps Spark scalar types to Postgres (STRING->TEXT, LONG->BIGINT, SHORT->SMALLINT, BYTE->SMALLINT), collapses nested ARRAY/MAP/STRUCT<...> to JSONB (flagged), maps TIMESTAMP (instant) to TIMESTAMPTZ, and drops the trailing table COMMENT.

## Run

```
mog -m databricks-tables-to-postgres <file>
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
CREATE TABLE sales.customers (
  id BIGINT,
  name TEXT,
  email TEXT,
  balance DECIMAL(18,2),
  tags JSONB, -- TODO(mog): collapsed from a Databricks nested type; element/field types were lost
  attrs JSONB, -- TODO(mog): collapsed from a Databricks nested type; element/field types were lost
  created_at TIMESTAMPTZ,
  region TEXT)
-- TODO(mog): dropped Databricks PARTITIONED BY (region) (set target partitioning / clustering manually)
-- TODO(mog): table comment dropped (use COMMENT ON TABLE in Postgres)
-- TODO(mog): dropped source LOCATION 's3://acme-lake/warehouse/sales.db/customers' (set target storage / external stage manually)
```

## Steps

- `run_mog`
- `replace_regex`: Reduce catalog.schema.table to schema.table (Postgres has no Unity catalog).
- `replace_regex`: Collapse nested ARRAY/MAP/STRUCT<...> to JSONB (flagged separately).
- `flag_matching`: Flag JSONB columns collapsed from a Databricks nested type.
- `replace_regex`: Spark TIMESTAMP (instant) -> TIMESTAMPTZ.
- `replace_map`: Map Spark scalar type keywords to Postgres.
- `replace_regex`: Drop the trailing table COMMENT (Postgres uses COMMENT ON).

## Tags

`sql` `databricks` `spark` `postgres` `convert` `migration`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

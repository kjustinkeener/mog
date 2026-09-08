# Databricks SHOW CREATE TABLE to MySQL

Convert a Databricks/Spark table export to MySQL DDL

Convert Databricks 'SHOW CREATE TABLE' output to MySQL. Strips Delta/Spark export noise, reduces catalog.schema.table to the bare table, collapses nested ARRAY/MAP/STRUCT to JSON (flagged), maps Spark scalars (STRING->TEXT, LONG->BIGINT, SHORT->SMALLINT, BYTE->TINYINT) and TIMESTAMP->DATETIME.

## Run

```
mog -m databricks-tables-to-mysql <file>
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
  name TEXT,
  email TEXT,
  balance DECIMAL(18,2),
  tags JSON, -- TODO(mog): collapsed from a Databricks nested type; element/field types were lost
  attrs JSON, -- TODO(mog): collapsed from a Databricks nested type; element/field types were lost
  created_at DATETIME,
  region TEXT)
-- TODO(mog): dropped Databricks PARTITIONED BY (region) (set target partitioning / clustering manually)
-- TODO(mog): table comment dropped (use COMMENT ON in MySQL)
-- TODO(mog): dropped source LOCATION 's3://acme-lake/warehouse/sales.db/customers' (set target storage / external stage manually)
```

## Steps

- `run_mog`
- `replace_regex`: Reduce catalog.schema.table to the bare table.
- `replace_regex`: Collapse nested ARRAY/MAP/STRUCT<...> to JSON (flagged).
- `flag_matching`: Flag JSON columns collapsed from a nested type.
- `replace_regex`: Spark TIMESTAMP -> DATETIME.
- `replace_map`: Map Spark scalar types to MySQL.
- `replace_regex`: Drop the trailing table COMMENT.

## Tags

`sql` `databricks` `spark` `mysql` `convert` `migration`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

# Databricks SHOW CREATE TABLE to Redshift

Convert a Databricks/Spark table export to Redshift DDL

Convert Databricks 'SHOW CREATE TABLE' output to Redshift. Strips Delta/Spark export noise, reduces catalog.schema.table to the bare table, collapses nested ARRAY/MAP/STRUCT to SUPER (flagged), maps Spark scalars (STRING->VARCHAR(65535), LONG->BIGINT, SHORT->SMALLINT, BYTE->SMALLINT) and TIMESTAMP->TIMESTAMP.

## Run

```
mog -m databricks-tables-to-redshift <file>
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
  name VARCHAR(65535),
  email VARCHAR(65535),
  balance DECIMAL(18,2),
  tags SUPER, -- TODO(mog): collapsed from a Databricks nested type; element/field types were lost
  attrs SUPER, -- TODO(mog): collapsed from a Databricks nested type; element/field types were lost
  created_at TIMESTAMP,
  region VARCHAR(65535))
-- TODO(mog): dropped Databricks PARTITIONED BY (region) (set target partitioning / clustering manually)
-- TODO(mog): table comment dropped (use COMMENT ON in Redshift)
-- TODO(mog): dropped source LOCATION 's3://acme-lake/warehouse/sales.db/customers' (set target storage / external stage manually)
```

## Pipeline

- `run_mog`
- `replace_regex`: Reduce catalog.schema.table to the bare table.
- `replace_regex`: Collapse nested ARRAY/MAP/STRUCT<...> to SUPER (flagged).
- `flag_matching`: Flag SUPER columns collapsed from a nested type.
- `replace_regex`: Spark TIMESTAMP -> TIMESTAMP.
- `replace_map`: Map Spark scalar types to Redshift.
- `replace_regex`: Drop the trailing table COMMENT.

## Tags

`sql` `databricks` `spark` `redshift` `convert` `migration`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

# Databricks SHOW CREATE TABLE to Snowflake

Convert a Databricks/Spark table export to Snowflake DDL

Convert the output of Databricks 'SHOW CREATE TABLE' to Snowflake. (1) Strips Delta/Spark export noise (USING, LOCATION, TBLPROPERTIES) and flags the physical clauses with no Snowflake equivalent, (2) rewrites Spark scalar types to Snowflake (STRING->VARCHAR, LONG->BIGINT, SHORT->SMALLINT, BYTE->TINYINT), (3) collapses nested ARRAY/MAP/STRUCT types to VARIANT and flags the lost element types, (4) flags bare TIMESTAMP (instant vs Snowflake's default NTZ wall-clock), and (5) rewrites CREATE TABLE to CREATE OR REPLACE TABLE and the table COMMENT. Not a semantic transpiler. Assumes one table's SHOW CREATE TABLE output.

## Run

```
mog -m databricks-tables-to-snowflake <file>
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
CREATE OR REPLACE TABLE main.sales.customers (
  id BIGINT,
  name VARCHAR,
  email VARCHAR,
  balance DECIMAL(18,2),
  tags ARRAY, -- TODO(mog): Snowflake ARRAY/OBJECT is untyped; the Databricks element/field types were dropped
  attrs OBJECT, -- TODO(mog): Snowflake ARRAY/OBJECT is untyped; the Databricks element/field types were dropped
  created_at TIMESTAMP_LTZ, -- NOTE(mog): mapped Databricks TIMESTAMP (UTC instant) to Snowflake TIMESTAMP_LTZ; use TIMESTAMP_NTZ instead if the source was wall-clock
  region VARCHAR)
-- TODO(mog): dropped Databricks PARTITIONED BY (region) (set target partitioning / clustering manually)
COMMENT = 'customer master table'
-- TODO(mog): dropped source LOCATION 's3://acme-lake/warehouse/sales.db/customers' (set target storage / external stage manually)
```

## Pipeline

- `run_mog`: Remove Delta/Spark boilerplate; flag dropped physical clauses.
- `run_mog`: Spark `id` backticks to Snowflake "id" double quotes.
- `replace`: Snowflake export idiom: CREATE TABLE -> CREATE OR REPLACE TABLE.
- `replace_regex`: Databricks ARRAY<...> -> Snowflake ARRAY (untyped; element type dropped).
- `replace_regex`: Databricks MAP<...> -> Snowflake OBJECT (untyped key/value).
- `replace_regex`: Databricks STRUCT<...> -> Snowflake OBJECT (field types dropped).
- `flag_matching`: Flag ARRAY/OBJECT columns: Snowflake's are untyped, so element/field types were lost.
- `replace_map`: Map Spark scalar types to Snowflake; Databricks TIMESTAMP (instant) -> TIMESTAMP_LTZ.
- `flag_matching`: Note the instant-preserving timestamp choice for review.
- `replace_regex`: Table-level COMMENT 'x' -> Snowflake COMMENT = 'x'.
- `run_mog`: Canonical whitespace tidy.

## Tags

`sql` `databricks` `spark` `snowflake` `warehouse` `convert` `migration`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

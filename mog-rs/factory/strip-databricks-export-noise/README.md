# Strip Databricks SHOW CREATE TABLE noise

Remove Delta/Spark export boilerplate, flag lost physical clauses

Strip the tool-specific boilerplate that Databricks 'SHOW CREATE TABLE' adds around a table definition, so a downstream dialect converter sees a clean CREATE TABLE. Removes the USING <format> directive and the whole TBLPROPERTIES(delta.*) block outright (engine internals), and turns the LOCATION / PARTITIONED BY / CLUSTER BY physical clauses into TODO(mog) comments (they carry intent but have no portable target). Target-agnostic: pair it with a per-target type/function converter. Assumes LF; run it first.

## Run

```
mog -m strip-databricks-export-noise <file>
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
CREATE TABLE main.sales.customers (
  id BIGINT,
  name STRING,
  email STRING,
  balance DECIMAL(18,2),
  tags ARRAY<STRING>,
  attrs MAP<STRING, STRING>,
  created_at TIMESTAMP,
  region STRING)
-- TODO(mog): dropped Databricks PARTITIONED BY (region) (set target partitioning / clustering manually)
COMMENT 'customer master table'
-- TODO(mog): dropped source LOCATION 's3://acme-lake/warehouse/sales.db/customers' (set target storage / external stage manually)
```

## Pipeline

- `eol_lf`: Normalize to LF so the block/line anchors below are reliable.
- `replace_regex`: Drop the entire TBLPROPERTIES(delta.*) block (Delta engine internals).
- `replace_regex`: Drop the USING <format> storage directive (no target equivalent).
- `replace_regex`: Turn the physical LOCATION clause into a TODO (Databricks-specific path).
- `replace_regex`: Turn PARTITIONED BY into a TODO (Hive-style partitioning has no portable target).
- `replace_regex`: Turn liquid CLUSTER BY into a TODO (set the target's clustering manually).

## Tags

`sql` `databricks` `spark` `strip` `migration` `fragment`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

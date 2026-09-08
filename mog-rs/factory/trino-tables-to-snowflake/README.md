# Trino to Snowflake

Convert a Trino table definition to Snowflake DDL

Convert Trino table DDL to Snowflake: JSON->VARIANT, TIMESTAMP->TIMESTAMP_NTZ, TIMESTAMP WITH TIME ZONE->TIMESTAMP_TZ, and CREATE TABLE->CREATE OR REPLACE TABLE; BIGINT/VARCHAR/DECIMAL pass through.

## Run

```
mog -m trino-tables-to-snowflake <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
CREATE TABLE customers (
  id BIGINT,
  name VARCHAR,
  balance DECIMAL(18,2),
  attrs JSON,
  created_at TIMESTAMP,
  updated_at TIMESTAMP WITH TIME ZONE
);
```

Output:

```
CREATE OR REPLACE TABLE customers (
  id BIGINT,
  name VARCHAR,
  balance DECIMAL(18,2),
  attrs VARIANT,
  created_at TIMESTAMP_NTZ,
  updated_at TIMESTAMP_TZ
);
```

## Steps

- `eol_lf`
- `replace_regex`
- `replace_regex`
- `replace_regex`
- `replace_regex`

## Tags

`sql` `trino` `snowflake` `convert` `migration`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

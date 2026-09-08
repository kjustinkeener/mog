# SQL cast: :: to CAST(... AS ...)

Rewrite SQL :: casts to CAST(expr AS type)

Rewrite the Postgres/Snowflake/Redshift shorthand cast expr::TYPE into the standard CAST(expr AS TYPE), for dialects that do not support :: (BigQuery, Databricks/Spark). Handles a simple operand (an identifier, dotted path, or parenthesized group) and an optional type length like (10) or (10,2). A cast whose operand is a larger expression needs manual review. Assumes LF text.

## Run

```
mog -m sql-colon-cast-to-cast <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
SELECT
  amount::NUMERIC(10,2),
  order_id::STRING,
  t.created_at::DATE,
  "created_at"::DATE,
  (a + b)::INT
FROM orders;
```

Output:

```
SELECT
  CAST(amount AS NUMERIC(10,2)),
  CAST(order_id AS STRING),
  CAST(t.created_at AS DATE),
  CAST("created_at" AS DATE),
  CAST((a + b) AS INT)
FROM orders;
```

## Steps

- `replace_regex`: (group)::TYPE -> CAST((group) AS TYPE)
- `replace_regex`: identifier::TYPE (bare, dotted, or quoted/backticked) -> CAST(identifier AS TYPE)

## Tags

`fragment` `sql` `bigquery` `databricks` `convert`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

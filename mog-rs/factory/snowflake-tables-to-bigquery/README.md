# Snowflake to BigQuery (DDL/SQL)

Convert Snowflake SQL to BigQuery

Rewrite Snowflake SQL toward BigQuery: double-quoted identifiers to backticks, :: casts to CAST(...), Snowflake type keywords to BigQuery types (NUMBER->NUMERIC, VARCHAR/STRING->STRING, TIMESTAMP_NTZ->DATETIME, VARIANT->JSON, ...), and safe 1:1 function renames (NVL->IFNULL, IFF->IF). Not a semantic transpiler: it does not reorder DATEADD/DATEDIFF args, translate LISTAGG/QUALIFY/semi-structured access, or catch type keywords that collide with column names. Assumes LF text.

## Run

```
mog -m snowflake-tables-to-bigquery <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
CREATE OR REPLACE TABLE "analytics"."orders" (
  "order_id" NUMBER(38,0),
  "customer_id" NUMBER,
  "amount" NUMBER(10,2),
  "created_at" TIMESTAMP_NTZ,
  "updated_at" TIMESTAMP_TZ,
  "notes" VARCHAR(500),
  "payload" VARIANT,
  "is_active" BOOLEAN
);

SELECT
  "order_id",
  NVL("amount", 0) AS amount,
  IFF("is_active", 1, 0) AS active_flag,
  "created_at"::DATE AS created_date
FROM "analytics"."orders";
```

Output:

```
CREATE OR REPLACE TABLE `analytics`.`orders` (
  `order_id` NUMERIC(38,0),
  `customer_id` NUMERIC,
  `amount` NUMERIC(10,2),
  `created_at` DATETIME,
  `updated_at` TIMESTAMP,
  `notes` STRING(500),
  `payload` JSON,
  `is_active` BOOL
);

SELECT
  `order_id`,
  IFNULL(`amount`, 0) AS amount,
  IF(`is_active`, 1, 0) AS active_flag,
  CAST(`created_at` AS DATE) AS created_date
FROM `analytics`.`orders`;
```

## Pipeline

- `run_mog`: Rewrite expr::TYPE to CAST(expr AS TYPE) (BigQuery has no ::). Run before requoting so quoted operands still start with a quote.
- `run_mog`: Snowflake "id" identifiers to BigQuery `id` backticks.
- `replace_map`: Map Snowflake type keywords to BigQuery types (whole-word, case-insensitive; longest match wins).
- `replace_map`: Safe 1:1 function renames (same argument shape).

## Tags

`sql` `snowflake` `bigquery` `warehouse` `convert` `migration`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

# SQL requote: double-quote to backtick

Rewrite SQL double-quoted identifiers to backticks

Rewrite standard-SQL double-quoted identifiers ("id") to BigQuery/Spark backtick identifiers (`id`). Composable fragment for any converter that targets BigQuery or Databricks. Assumes the source uses single quotes for string literals (Snowflake/Postgres/Redshift), so every "..." is an identifier; a double quote inside a single-quoted string literal is a known limitation (rare in DDL). Assumes LF text.

## Run

```
mog -m sql-requote-quote-to-backtick <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
SELECT "id", "user name", "amount"
FROM "sales schema"."orders"
WHERE "status" = 'active';
```

Output:

```
SELECT `id`, `user name`, `amount`
FROM `sales schema`.`orders`
WHERE `status` = 'active';
```

## Pipeline

- `replace_regex`: "identifier" -> `identifier`

## Tags

`fragment` `sql` `bigquery` `databricks` `identifier` `quote`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

# SQL requote: backtick to double-quote

Rewrite SQL backtick identifiers to double quotes

Rewrite BigQuery/Spark backtick identifiers (`id`) to standard-SQL double-quoted identifiers ("id"). Composable fragment for any converter that targets Snowflake/Postgres/Redshift. Backticks only ever quote identifiers, so this is a safe mechanical rewrite. Assumes LF text.

## Run

```
mog -m sql-requote-backtick-to-quote <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
SELECT `id`, `user name`, `amount`
FROM `project.dataset.orders`
WHERE `status` = 'active';
```

Output:

```
SELECT "id", "user name", "amount"
FROM "project.dataset.orders"
WHERE "status" = 'active';
```

## Pipeline

- `replace_regex`: `identifier` -> "identifier"

## Tags

`fragment` `sql` `snowflake` `postgres` `identifier` `quote`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

# MySQL to BigQuery (DDL/SQL)

Convert MySQL SQL to BigQuery

Rewrite MySQL SQL toward BigQuery. Both use backtick identifiers, so those pass through; AUTO_INCREMENT is dropped (BigQuery has no auto-increment) and type keywords change (INT/TINYINT/SMALLINT/BIGINT->INT64, VARCHAR/TEXT->STRING, DOUBLE->FLOAT64, DECIMAL->NUMERIC, DATETIME->DATETIME, TIMESTAMP->TIMESTAMP, BLOB->BYTES). Minimal-diff, NOT a semantic transpiler. Assumes LF text.

## Run

```
mog -m mysql-tables-to-bigquery <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
CREATE TABLE `users` (
  `id` INT AUTO_INCREMENT,
  `name` VARCHAR(100),
  `bio` TEXT,
  `score` DOUBLE,
  `photo` BLOB
);
```

Output:

```
CREATE TABLE `users` (
  `id` INT64,
  `name` STRING(100),
  `bio` STRING,
  `score` FLOAT64,
  `photo` BYTES
);
```

## Pipeline

- `replace_regex`: Drop AUTO_INCREMENT (with its leading space)
- `replace_map`

## Tags

`sql` `mysql` `bigquery` `warehouse` `convert` `migration`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

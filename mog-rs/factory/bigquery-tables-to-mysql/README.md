# BigQuery to MySQL (DDL/SQL)

Convert BigQuery SQL to MySQL

Convert BigQuery Standard SQL toward MySQL: type maps (INT64->BIGINT, FLOAT64->DOUBLE, NUMERIC->DECIMAL, STRING->TEXT, BOOL->TINYINT(1), BYTES->BLOB, DATETIME stays) and SAFE_CAST->CAST. Both dialects quote identifiers with backticks, so those pass through. Minimal-diff, not a semantic transpiler: STRUCT/ARRAY types and BigQuery-only functions are left for review. Assumes LF text.

## Run

```
mog -m bigquery-tables-to-mysql <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
CREATE TABLE `users` (
  id INT64,
  name STRING,
  score FLOAT64,
  active BOOL,
  data BYTES
);
SELECT SAFE_CAST(score AS INT64) FROM `users`;
```

Output:

```
CREATE TABLE `users` (
  id BIGINT,
  name TEXT,
  score DOUBLE,
  active TINYINT(1),
  data BLOB
);
SELECT CAST(score AS BIGINT) FROM `users`;
```

## Steps

- `replace_map`: Map BigQuery types to MySQL (whole-word, case-insensitive)
- `replace_regex`: SAFE_CAST -> CAST

## Tags

`sql` `bigquery` `mysql` `warehouse` `convert` `migration`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

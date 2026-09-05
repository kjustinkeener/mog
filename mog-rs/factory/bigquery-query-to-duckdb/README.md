# BigQuery query to DuckDB

Rewrite BigQuery SELECT function calls for DuckDB

Convert common BigQuery scalar-function differences to DuckDB for simple (non-nested-argument) call shapes. Handles DATE_ADD(expr, INTERVAL n part) -> (expr + INTERVAL n part), DATE_DIFF(end, start, part) -> date_diff('part', start, end) (BigQuery's operand order is reversed), SAFE_CAST -> TRY_CAST, IFNULL -> COALESCE, and the INT64/FLOAT64/STRING type names inside casts. DATE_ADD/DATE_DIFF with nested-call arguments is flagged, not rewritten. Not a semantic transpiler.

## Run

```
mog -m bigquery-query-to-duckdb <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
SELECT
  id,
  DATE_ADD(created_at, INTERVAL 7 DAY) AS due_date,
  DATE_DIFF(shipped_at, created_at, DAY) AS days_to_ship,
  SAFE_CAST(amount AS INT64) AS amount_int,
  IFNULL(note, 'none') AS note
FROM orders
WHERE DATE_DIFF(shipped_at, created_at, DAY) > 3
ORDER BY id;
```

Output:

```
SELECT
  id,
  (created_at + INTERVAL 7 DAY) AS due_date,
  date_diff('DAY', created_at, shipped_at) AS days_to_ship,
  TRY_CAST(amount AS BIGINT) AS amount_int,
  COALESCE(note, 'none') AS note
FROM orders
WHERE date_diff('DAY', created_at, shipped_at) > 3
ORDER BY id;
```

## Pipeline

- `replace_regex`: DATE_ADD(expr, INTERVAL n part) -> (expr + INTERVAL n part). Simple args only.
- `replace_regex`: DATE_DIFF(end, start, part) -> date_diff('part', start, end). Note reversed operands.
- `replace`: SAFE_CAST -> TRY_CAST.
- `replace_regex`: IFNULL -> COALESCE.
- `replace_map`: GoogleSQL type names inside casts -> DuckDB.
- `flag_matching`: Flag any DATE_ADD/DATE_DIFF left unconverted (nested arguments). Case-sensitive: the converted form is lowercase date_diff.

## Tags

`sql` `bigquery` `duckdb` `query` `convert` `migration`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

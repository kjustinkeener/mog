# Snowflake query to DuckDB

Rewrite Snowflake SELECT function calls for DuckDB

Rewrite the common Snowflake scalar-function differences DuckDB does not share, for simple (non-nested-argument) call shapes. Handles DATEADD(part, n, expr) -> (expr + INTERVAL n part), DATEDIFF(part, a, b) -> date_diff('part', a, b), and NVL -> COALESCE; the :: cast passes through (DuckDB supports it). Any DATEADD/DATEDIFF whose arguments contain nested calls or commas is not rewritten (regex cannot balance parentheses) and is flagged with TODO(mog) for manual review. Not a semantic transpiler.

## Run

```
mog -m snowflake-query-to-duckdb <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
SELECT
  id,
  DATEADD(day, 7, created_at) AS due_date,
  DATEDIFF(day, created_at, shipped_at) AS days_to_ship,
  NVL(note, 'none') AS note,
  amount::INT AS amount_int
FROM orders
WHERE DATEADD(day, 30, created_at) >= created_at
ORDER BY id;
```

Output:

```
SELECT
  id,
  (created_at + INTERVAL 7 day) AS due_date,
  date_diff('day', created_at, shipped_at) AS days_to_ship,
  COALESCE(note, 'none') AS note,
  amount::INT AS amount_int
FROM orders
WHERE (created_at + INTERVAL 30 day) >= created_at
ORDER BY id;
```

## Pipeline

- `replace_regex`: DATEADD(part, n, col) -> (col + INTERVAL n part). Simple args only.
- `replace_regex`: DATEDIFF(part, a, b) -> date_diff('part', a, b) (DuckDB needs the part quoted).
- `replace_regex`: NVL -> COALESCE.
- `flag_matching`: Flag any DATEADD/DATEDIFF left unconverted (nested arguments; regex cannot balance parens).

## Tags

`sql` `snowflake` `duckdb` `query` `convert` `migration`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

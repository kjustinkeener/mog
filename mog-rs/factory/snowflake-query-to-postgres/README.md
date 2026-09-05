# Snowflake query to Postgres

Rewrite Snowflake SELECT function calls for PostgreSQL

Rewrite the common Snowflake scalar-function differences targeting PostgreSQL, for simple (non-nested-argument) call shapes. Handles DATEADD(part, n, expr) -> (expr + INTERVAL 'n part') (Postgres requires the quoted interval literal) and NVL -> COALESCE; the :: cast passes through (Postgres supports it). DATEDIFF is flagged, not converted: Postgres has no part-aware DATEDIFF (for day diffs on dates use end - start, otherwise EXTRACT/AGE). Any DATEADD with nested-call arguments is also flagged with TODO(mog). Not a semantic transpiler.

## Run

```
mog -m snowflake-query-to-postgres <file>
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
  (created_at + INTERVAL '7 day') AS due_date,
  DATEDIFF(day, created_at, shipped_at) AS days_to_ship, -- TODO(mog): Postgres has no part-aware DATEDIFF; for day diffs on dates use (end - start), else compute via EXTRACT / AGE
  COALESCE(note, 'none') AS note,
  amount::INT AS amount_int
FROM orders
WHERE (created_at + INTERVAL '30 day') >= created_at
ORDER BY id;
```

## Pipeline

- `replace_regex`: DATEADD(part, n, col) -> (col + INTERVAL 'n part'). Postgres needs the quoted interval.
- `replace_regex`: NVL -> COALESCE.
- `flag_matching`: Flag DATEDIFF (no part-aware Postgres equivalent).
- `flag_matching`: Flag any DATEADD left unconverted (nested arguments).

## Tags

`sql` `snowflake` `postgres` `query` `convert` `migration`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

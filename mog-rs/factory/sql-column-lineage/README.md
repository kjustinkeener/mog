# Trace SQL column lineage

Report each SELECT column's upstream source columns

For a single SELECT statement, report where each output column comes from: one line per projected column, 'output <- source.column, ...', listing the upstream table columns it derives from. Aliases are honored (SELECT o.id AS oid -> 'oid <- orders.id'), an unnamed expression is labeled col1, col2, ... by position, and a constant or otherwise unresolved projection shows '(none)'. Parse-based (polyglot-sql), so it understands joins and table aliases rather than matching text. The whole input is treated as one query; give it one statement at a time. This does not change the SQL, it summarizes it, so use it to review or document a query, not to rewrite it.

## Run

```
mog -m sql-column-lineage <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
SELECT o.id AS order_id, c.name AS customer, o.total, 'paid' AS status FROM orders o JOIN customers c ON o.customer_id = c.id
```

Output:

```
order_id <- orders.id
customer <- customers.name
total <- orders.total
status <- (none)
```

## Pipeline

- `sql_lineage`: Emit 'output <- source.column' for each projected column

## Tags

`sql` `lineage` `analyze` `single-action`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

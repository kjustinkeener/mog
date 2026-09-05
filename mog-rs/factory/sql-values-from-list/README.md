# List to SQL VALUES tuples

Build SQL VALUES tuples from a list

Turn a plain list (one value per line) into a comma-joined SQL VALUES list like ('a'),('b'),('c'), ready to drop into a bulk INSERT. Each value is SQL-escaped (embedded quotes doubled) and wrapped in ('...'). Blank lines are dropped. For a single-column IN (...) list instead, see sql-in-clause.

## Run

```
mog -m sql-values-from-list <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
apple
banana
O'Brien
```

Output:

```
('apple'),('banana'),('O''Brien')
```

## Pipeline

- `remove_empty_lines`: Drop blank lines
- `escape_sql`: SQL-quote each value
- `wrap_lines`: Wrap each as a single-column tuple
- `join_lines`: Comma-join the tuples

## Tags

`sql` `json` `list` `convert`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

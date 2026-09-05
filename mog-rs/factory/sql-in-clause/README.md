# SQL IN clause from a list

Build a SQL IN (...) clause from a list

Turn a plain list (one value per line) into a SQL IN (...) clause: trim each value, drop blank lines, single-quote each value with embedded single quotes doubled (SQL-escaped), join with ', ', and wrap in IN (...). Values are treated as string literals; numbers come out quoted too (most databases accept that, but drop the quotes yourself if you need bare numerics).

## Run

```
mog -m sql-in-clause <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
apple
  banana

O'Brien
cherry
```

Output:

```
IN ('apple', 'banana', 'O''Brien', 'cherry')
```

## Pipeline

- `trim_whitespace`: Trim surrounding whitespace from each value
- `remove_empty_lines`: Drop blank lines
- `replace`: SQL-escape embedded single quotes (' -> '')
- `wrap_lines`: Quote each value
- `join_lines`: Comma-separate the quoted values
- `replace`: Drop the trailing newline so ) hugs the last value
- `prepend`: Open the IN clause
- `append`: Close the IN clause

## Tags

`sql` `list` `quote`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

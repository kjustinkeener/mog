# Markdown table to CSV

Convert a Markdown table to CSV

Parse a Markdown table into CSV, dropping the separator row. Cells are trimmed and CSV-quoted as needed; non-table lines are ignored. The read counterpart to CSV to Markdown table.

## Run

```
mog -m markdown-table-to-csv <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
| name | role |
| --- | --- |
| Ada | Engineer |
| Bo | Manager |
```

Output:

```
name,role
Ada,Engineer
Bo,Manager
```

## Pipeline

- `markdown_table_to_csv`: Parse the Markdown table into CSV rows.

## Tags

`convert` `markdown` `csv` `table`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

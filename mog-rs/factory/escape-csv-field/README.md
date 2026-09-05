# Escape each line as a CSV field

Quote each line as one RFC-4180 CSV field

Quote each line as a single RFC-4180 CSV field: a value containing a comma, quote, or newline is wrapped in double quotes with embedded quotes doubled. Use before pasting free text into a CSV column so delimiters do not break the row.

## Run

```
mog -m escape-csv-field <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
plain value
has,a,comma
has "quotes"
```

Output:

```
plain value
"has,a,comma"
"has ""quotes"""
```

## Pipeline

- `escape_csv`: Quote each line as a CSV field

## Tags

`escape` `csv` `encode` `quote` `data` `single-action`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

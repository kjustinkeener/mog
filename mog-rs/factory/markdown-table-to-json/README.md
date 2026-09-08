# Markdown table to JSON

Convert a Markdown table to a JSON array of objects

Convert a GitHub-flavored Markdown table into a JSON array of objects, one object per row keyed by the header cells. The --- separator row is dropped and escaped pipes are decoded.

## Run

```
mog -m markdown-table-to-json <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
| id | name | role |
| --- | --- | --- |
| 1 | Ada | Lead |
| 2 | Bob | Eng |
```

Output:

```
[{"id":"1","name":"Ada","role":"Lead"},{"id":"2","name":"Bob","role":"Eng"}]
```

## Steps

- `markdown_table_to_csv`: Parse the Markdown table into CSV
- `csv_to_json`: Turn the rows into a JSON array of objects

## Tags

`markdown` `table` `json` `convert` `data` `docs`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

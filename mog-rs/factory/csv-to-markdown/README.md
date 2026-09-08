# CSV to Markdown table

Convert CSV to a Markdown table

Render CSV (with a header row) as a GitHub-flavored Markdown table. Quote-aware; a pipe inside a cell is escaped; ragged rows are padded to the header width.

## Run

```
mog -m csv-to-markdown <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
name,role
Ada,Engineer
"Bo, Jr",Manager
```

Output:

```
| name | role |
| --- | --- |
| Ada | Engineer |
| Bo, Jr | Manager |
```

## Steps

- `csv_to_markdown`: Render the CSV as a Markdown table.

## Tags

`convert` `csv` `markdown` `table`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

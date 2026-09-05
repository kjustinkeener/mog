# TSV to Markdown table

Convert TSV to a Markdown table

Render a tab-separated table (first line = header) as a GitHub-flavored Markdown table. A pipe inside a cell is escaped; ragged rows are padded to the header width. Handy for pasting spreadsheet data straight into a Markdown doc or PR.

## Run

```
mog -m tsv-to-markdown <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
name	role	team
Ada	Lead	Platform
Bob	Eng	Data
```

Output:

```
| name | role | team |
| --- | --- | --- |
| Ada | Lead | Platform |
| Bob | Eng | Data |
```

## Pipeline

- `csv_to_markdown`: Read the input as tab-delimited and emit a Markdown table

## Tags

`tsv` `markdown` `table` `convert` `docs` `data`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

# Markdown table to TSV

Convert a Markdown table to TSV

Parse a GitHub-flavored Markdown table into tab-separated values, dropping the --- separator row. Cells are trimmed; an escaped pipe becomes a literal pipe. Non-table lines are ignored. The inverse of tsv-to-markdown; use it to get spreadsheet-pasteable data back out of a Markdown doc.

## Run

```
mog -m markdown-table-to-tsv <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
| name | role | team |
| --- | --- | --- |
| Ada | Lead | Platform |
| Bob | Eng | Data |
```

Output:

```
name	role	team
Ada	Lead	Platform
Bob	Eng	Data
```

## Pipeline

- `markdown_table_to_csv`: Emit the table as tab-delimited rows

## Tags

`markdown` `table` `tsv` `convert` `docs` `data`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

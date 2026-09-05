# Markdown table to YAML

Convert a Markdown table to a YAML list

Convert a GitHub-flavored Markdown table into a YAML list of mappings, one entry per row keyed by the header cells. Values are kept as strings.

## Run

```
mog -m markdown-table-to-yaml <file>
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
- id: '1'
  name: Ada
  role: Lead
- id: '2'
  name: Bob
  role: Eng
```

## Pipeline

- `markdown_table_to_csv`: Parse the table into CSV
- `csv_to_json`: Rows to a JSON array of objects
- `json_to_yaml`: Serialize as YAML

## Tags

`markdown` `table` `yaml` `convert` `docs` `data`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

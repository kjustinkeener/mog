# TSV to CSV

Convert TSV to CSV with RFC 4180 quoting

Convert tab-separated values to comma-separated values, quote-aware: any field that contains a comma, a quote, or a CR/LF is wrapped in double quotes (RFC 4180), so a cell like London, UK is preserved as "London, UK". The inverse of csv-to-tsv. Single-line fields only.

## Run

```
mog -m tsv-to-csv <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
name	city	note
Ada	London, UK	first
Bob	Paris	second
```

Output:

```
name,city,note
Ada,"London, UK",first
Bob,Paris,second
```

## Pipeline

- `change_delimiter`: Re-delimit each line from tab to comma (re-quoting as needed)

## Tags

`tsv` `csv` `convert` `data` `fragment`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

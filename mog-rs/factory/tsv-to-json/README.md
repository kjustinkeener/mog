# TSV to JSON

Convert TSV to a JSON array of objects

Convert a tab-separated table (first line = header) into a JSON array of objects, one object per data row keyed by the header names. Quote-aware. Values are kept as strings by default.

## Run

```
mog -m tsv-to-json <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
id	name	role
1	Ada	Lead
2	Bob	Eng
```

Output:

```
[{"id":"1","name":"Ada","role":"Lead"},{"id":"2","name":"Bob","role":"Eng"}]
```

## Pipeline

- `csv_to_json`: Read tab-delimited rows into a JSON array of objects

## Tags

`tsv` `json` `convert` `data`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

# JSON to CSV

Convert JSON or JSONL to CSV

Convert a JSON array (or JSONL, one object per line) of flat objects to CSV. The header is the union of keys in first-seen order, missing keys are empty cells, and nested values become compact JSON. The read counterpart to CSV to JSON.

## Run

```
mog -m json-to-csv <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
{"id":1,"name":"Ada"}
{"id":2,"city":"NYC"}
{"id":3,"name":"Cy","city":"LA"}
```

Output:

```
id,name,city
1,Ada,
2,,NYC
3,Cy,LA
```

## Pipeline

- `json_to_csv`: Flatten the JSON objects into CSV rows.

## Tags

`convert` `json` `csv` `data`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

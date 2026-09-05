# JSONL to CSV

Convert JSONL to CSV

Convert JSON Lines (one JSON object per line) into a CSV table: collect the objects into an array, then flatten to CSV with a header row (the union of keys). Nested objects/arrays are rendered as compact JSON in their cell.

## Run

```
mog -m jsonl-to-csv <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
{"id": 1, "name": "Ada", "role": "Lead"}
{"id": 2, "name": "Bob", "role": "Eng"}
{"id": 3, "name": "Cy", "role": "Eng"}
```

Output:

```
id,name,role
1,Ada,Lead
2,Bob,Eng
3,Cy,Eng
```

## Pipeline

- `jsonl_to_json`: Gather the JSONL objects into a JSON array
- `json_to_csv`: Flatten the array of objects to CSV

## Tags

`jsonl` `json` `csv` `convert` `data`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

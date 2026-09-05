# CSV to JSON

Convert CSV to a JSON array of objects

Convert CSV (with a header row) to a JSON array of objects keyed by the header. Types are inferred: numbers, true/false, and empty cells become null. Quote-aware but single-line (no newlines inside quoted fields).

## Run

```
mog -m csv-to-json <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
id,name,active
1,Ada,true
2,"Bo, Jr",false
3,Cy,true
```

Output:

```
[{"id":1,"name":"Ada","active":true},{"id":2,"name":"Bo, Jr","active":false},{"id":3,"name":"Cy","active":true}]
```

## Pipeline

- `csv_to_json`: Read the CSV into a typed JSON array.

## Tags

`convert` `csv` `json` `data`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

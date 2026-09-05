# CSV to YAML

Convert CSV to a YAML list of objects

Convert CSV (with a header row) to a YAML list of objects, keyed by the header, with types inferred.

## Run

```
mog -m csv-to-yaml <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
id,name
1,Ada
2,Bo
```

Output:

```
- id: 1
  name: Ada
- id: 2
  name: Bo
```

## Pipeline

- `csv_to_json`: Read the CSV into a typed JSON array.
- `json_to_yaml`: Render the JSON array as YAML.

## Tags

`convert` `csv` `yaml` `data`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

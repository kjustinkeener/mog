# YAML to CSV

Convert YAML to CSV

Convert a YAML list of flat objects to CSV. The header is the union of keys; missing keys are empty cells.

## Run

```
mog -m yaml-to-csv <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
- id: 1
  name: Ada
- id: 2
  name: Bo
```

Output:

```
id,name
1,Ada
2,Bo
```

## Steps

- `yaml_to_json`: Parse the YAML list into a compact JSON array.
- `json_to_csv`: Flatten the JSON array into CSV.

## Tags

`convert` `yaml` `csv` `data`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

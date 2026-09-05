# JSONL to YAML

Convert JSONL to a YAML list

Convert JSON Lines (one JSON object per line) into a YAML list: gather the objects into an array, then serialize as YAML. Key order is preserved.

## Run

```
mog -m jsonl-to-yaml <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
{"id": 1, "name": "Ada"}
{"id": 2, "name": "Bob"}
```

Output:

```
- id: 1
  name: Ada
- id: 2
  name: Bob
```

## Pipeline

- `jsonl_to_json`: Gather JSONL into a JSON array
- `json_to_yaml`: Serialize as YAML

## Tags

`jsonl` `json` `yaml` `convert` `data`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

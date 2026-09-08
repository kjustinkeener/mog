# YAML to JSONL

Convert YAML to JSONL

Convert a YAML list of objects into JSON Lines (one compact JSON object per line). Best for a top-level YAML sequence; key order is preserved.

## Run

```
mog -m yaml-to-jsonl <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
- id: 1
  name: Ada
- id: 2
  name: Bob
```

Output:

```
{"id":1,"name":"Ada"}
{"id":2,"name":"Bob"}
```

## Steps

- `yaml_to_json`: Parse YAML into a JSON array
- `json_to_jsonl`: Emit one object per line

## Tags

`yaml` `json` `jsonl` `convert` `data`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

# YAML to JSON

Convert YAML to JSON

Parse a YAML document and emit it as pretty JSON, with key order preserved. Single-document YAML.

## Run

```
mog -m yaml-to-json <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
name: App
port: 8080
tags:
  - a
  - b
```

Output:

```
{
  "name": "App",
  "port": 8080,
  "tags": [
    "a",
    "b"
  ]
}
```

## Steps

- `yaml_to_json`: Convert the YAML document to JSON.

## Tags

`convert` `yaml` `json` `data`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

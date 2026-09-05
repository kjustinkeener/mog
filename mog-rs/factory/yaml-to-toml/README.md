# YAML to TOML

Convert YAML to TOML

Convert a YAML document to TOML, bridging through JSON. Best for a YAML mapping (an object at the top level); key order is preserved. Anchors/aliases and multi-document streams are not supported.

## Run

```
mog -m yaml-to-toml <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
name: demo
version: 2
server:
  host: localhost
  port: 8080
tags:
  - a
  - b
```

Output:

```
name = "demo"
version = 2
tags = [
    "a",
    "b",
]

[server]
host = "localhost"
port = 8080
```

## Pipeline

- `yaml_to_json`: Parse YAML into JSON
- `json_to_toml`: Serialize the JSON as TOML

## Tags

`yaml` `toml` `config` `convert`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

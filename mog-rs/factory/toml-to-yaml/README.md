# TOML to YAML

Convert TOML to YAML

Convert a TOML document to YAML, bridging through JSON. Key order is preserved. The inverse of yaml-to-toml.

## Run

```
mog -m toml-to-yaml <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
name = "demo"
version = 2

[server]
host = "localhost"
port = 8080
```

Output:

```
name: demo
version: 2
server:
  host: localhost
  port: 8080
```

## Pipeline

- `toml_to_json`: Parse TOML into JSON
- `json_to_yaml`: Serialize the JSON as YAML

## Tags

`toml` `yaml` `config` `convert`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

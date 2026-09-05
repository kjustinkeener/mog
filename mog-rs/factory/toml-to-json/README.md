# TOML to JSON

Convert TOML to JSON

Parse TOML and emit it as pretty JSON, with key order preserved. Pairs with JSON to TOML.

## Run

```
mog -m toml-to-json <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
title = "App"
port = 8080

[db]
host = "localhost"
pool = 5
```

Output:

```
{
  "title": "App",
  "port": 8080,
  "db": {
    "host": "localhost",
    "pool": 5
  }
}
```

## Pipeline

- `toml_to_json`: Convert the TOML to JSON.

## Tags

`convert` `toml` `json` `config`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

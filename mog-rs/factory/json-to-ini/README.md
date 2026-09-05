# JSON to INI

Convert a JSON object to INI config

Convert a JSON object to INI. Top-level scalars come first, then each nested object as an [section]. Best for the flat/one-level shape; deeply nested structures do not map cleanly to INI.

## Run

```
mog -m json-to-ini <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
{
  "app": "demo",
  "server": {
    "host": "localhost",
    "port": 8080
  }
}
```

Output:

```
app = demo

[server]
host = localhost
port = 8080
```

## Pipeline

- `json_to_toml`: Serialize JSON as TOML
- `toml_to_ini`: Convert the TOML to INI

## Tags

`json` `ini` `config` `convert`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

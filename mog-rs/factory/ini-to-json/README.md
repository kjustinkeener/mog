# INI to JSON

Convert INI to JSON

Convert an INI file to JSON. [section] headers become nested objects and key=value lines get type-inferred values. Best for the common flat/one-level INI shape.

## Run

```
mog -m ini-to-json <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
app = demo

[server]
host = localhost
port = 8080
```

Output:

```
{
  "app": "demo",
  "server": {
    "host": "localhost",
    "port": 8080
  }
}
```

## Pipeline

- `ini_to_toml`: Parse INI into TOML
- `toml_to_json`: Serialize as JSON

## Tags

`ini` `json` `config` `convert`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

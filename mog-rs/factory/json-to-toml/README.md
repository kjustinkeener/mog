# JSON to TOML

Convert a JSON object to TOML

Parse a JSON object and emit it as TOML. The top level must be an object; JSON null has no TOML equivalent. The read counterpart to TOML to JSON.

## Run

```
mog -m json-to-toml <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
{"title": "App", "port": 8080, "db": {"host": "localhost", "pool": 5}}
```

Output:

```
title = "App"
port = 8080

[db]
host = "localhost"
pool = 5
```

## Pipeline

- `json_to_toml`: Convert the JSON object to TOML.

## Tags

`convert` `json` `toml` `config`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

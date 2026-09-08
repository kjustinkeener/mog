# JSON to YAML

Convert JSON to YAML

Parse a JSON value and emit it as YAML. The read counterpart to YAML to JSON.

## Run

```
mog -m json-to-yaml <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
{"name":"App","port":8080,"tags":["a","b"]}
```

Output:

```
name: App
port: 8080
tags:
- a
- b
```

## Steps

- `json_to_yaml`: Convert the JSON value to YAML.

## Tags

`convert` `json` `yaml` `data`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

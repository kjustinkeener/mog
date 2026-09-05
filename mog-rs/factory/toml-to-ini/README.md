# TOML to INI

Convert TOML to INI

Convert a TOML config to INI: top-level scalars first, then each table as a [section] (nested tables become dotted sections), with arrays comma-joined. Handles the flat/one-level shape. The inverse of INI to TOML.

## Run

```
mog -m toml-to-ini <file>
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
port = 8080
title = App

[db]
host = localhost
pool = 5
```

## Pipeline

- `toml_to_ini`: Convert the TOML config to INI.

## Tags

`convert` `config` `toml` `ini`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

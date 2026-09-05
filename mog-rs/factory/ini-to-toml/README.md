# INI to TOML

Convert INI to TOML

Convert an INI config to TOML. [section] headers pass through, key=value lines get a type-inferred TOML value (bool/int/float, else a quoted string), and ;/# comment lines become TOML # comments. Best for the common flat/one-level shape.

## Run

```
mog -m ini-to-toml <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
; app config
title = App
port = 8080

[db]
host = localhost
debug = true
```

Output:

```
# app config
title = "App"
port = 8080

[db]
host = "localhost"
debug = true
```

## Pipeline

- `ini_to_toml`: Convert the INI config to TOML.

## Tags

`convert` `config` `ini` `toml`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

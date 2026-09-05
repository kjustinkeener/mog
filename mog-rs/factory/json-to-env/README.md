# JSON to .env

Convert a flat JSON object to KEY=VALUE .env lines

Convert a flat JSON object to KEY=VALUE .env lines. Values with spaces or special characters are double-quoted; null is skipped. The read counterpart to .env to JSON.

## Run

```
mog -m json-to-env <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
{"NAME": "App", "URL": "http://example.com/x", "PORT": 8080, "NOTE": "a b"}
```

Output:

```
NAME=App
URL=http://example.com/x
PORT=8080
NOTE="a b"
```

## Pipeline

- `json_to_env`: Render the JSON object as .env lines.

## Tags

`convert` `json` `env` `dotenv` `config`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

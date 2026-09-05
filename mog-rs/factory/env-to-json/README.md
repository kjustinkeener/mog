# .env to JSON

Convert .env to JSON

Parse a .env file (KEY=VALUE, # comments, optional export, optional surrounding quotes) into a flat JSON object of string values. Pairs with JSON to .env.

## Run

```
mog -m env-to-json <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
# app config
export NAME=App
URL="http://example.com/x"
PORT=8080
```

Output:

```
{
  "NAME": "App",
  "URL": "http://example.com/x",
  "PORT": "8080"
}
```

## Pipeline

- `env_to_json`: Parse the .env into a JSON object.

## Tags

`convert` `env` `dotenv` `json` `config`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

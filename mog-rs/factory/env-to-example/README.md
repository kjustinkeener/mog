# Make .env.example (blank the values)

Make a .env.example by blanking values

Produce a .env.example from a .env by blanking every value while keeping the keys, so you can commit a template without leaking secrets. Each KEY=value line becomes KEY=; comment (#) and blank lines pass through unchanged.

## Run

```
mog -m env-to-example <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
# app secrets
API_KEY=sk-live-abc123
DB_PASSWORD=hunter2
PORT=8080
```

Output:

```
# app secrets
API_KEY=
DB_PASSWORD=
PORT=
```

## Pipeline

- `replace_regex_multiline`: Blank the value after the first = on each key line

## Tags

`env` `dotenv` `docs` `secrets` `config`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

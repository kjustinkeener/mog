# dotenv to shell exports

Convert .env to shell export statements

Turn a .env file into shell `export KEY=VALUE` statements you can source. Only KEY=VALUE lines are prefixed with `export `; comments (#) and blank lines pass through untouched. Values are left exactly as written (no quoting changes), so quote them in the .env if they contain spaces.

## Run

```
mog -m dotenv-to-exports <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
# app config
PORT=8080
HOST=localhost

# database
DB_URL=postgres://localhost/app
```

Output:

```
# app config
export PORT=8080
export HOST=localhost

# database
export DB_URL=postgres://localhost/app
```

## Steps

- `prefix_lines`: Prefix each KEY=VALUE line with 'export '

## Tags

`env` `dotenv` `shell` `convert` `config`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

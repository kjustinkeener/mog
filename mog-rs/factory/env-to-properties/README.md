# .env to Java properties

Convert .env keys to Java properties style

Convert a .env body to Java .properties style: on each KEY=VALUE line, lowercase the key and turn underscores into dots (APP_SERVER_PORT=8080 -> app.server.port=8080). The value (right of the first =) is left as written. Comment (#) and blank lines are untouched. The inverse of properties-to-env.

## Run

```
mog -m env-to-properties <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
# App configuration

APP_SERVER_PORT=8080
LOG_LEVEL=debug
FEATURE_ENABLED=true
```

Output:

```
# App configuration

app.server.port=8080
log.level=debug
feature.enabled=true
```

## Pipeline

- `to_lower`: Lowercase the key (left of =)
- `replace`: Underscores to dots in the key

## Tags

`env` `dotenv` `config` `convert` `java`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

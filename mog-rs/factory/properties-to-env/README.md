# Java properties to .env

Convert Java .properties to .env format

Convert a Java .properties body to .env style: on each key=value line, uppercase the key and turn dots into underscores (app.server.port=8080 -> APP_SERVER_PORT=8080). The value (right of the first =) is left exactly as written. Only the key is changed; comment (# / !) and blank lines are left untouched.

## Run

```
mog -m properties-to-env <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
# App configuration

app.server.port=8080
log.level=debug
feature.enabled=true
```

Output:

```
# App configuration

APP_SERVER_PORT=8080
LOG_LEVEL=debug
FEATURE_ENABLED=true
```

## Steps

- `to_upper`: Uppercase the key (left of =)
- `replace`: Dots to underscores in the key

## Tags

`config` `env` `dotenv` `convert` `java`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

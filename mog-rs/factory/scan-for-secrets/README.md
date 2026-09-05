# Flag lines that look like secrets

Tag lines containing an API-key-shaped token

Flag each line that looks like it contains a secret (AWS/Google/Stripe/GitHub-shaped key or token) by appending an inline comment tag. A non-destructive audit for logs and configs; pair with redact-secrets to act on the hits.

## Run

```
mog -m scan-for-secrets <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
aws_key = AKIAIOSFODNN7EXAMPLE
db_host = localhost
port = 5432
```

Output:

```
aws_key = AKIAIOSFODNN7EXAMPLE # WARN(mog): possible secret (aws-key)
db_host = localhost
port = 5432
```

## Pipeline

- `detect_secrets`: Tag lines that look like secrets

## Tags

`scan` `secrets` `audit` `security`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

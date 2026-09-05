# Flag lines that look like PII

Tag lines containing email/SSN/card-shaped data

Flag each line that looks like it contains shaped PII (email, SSN, credit card, phone) by appending an inline comment tag. A non-destructive audit: it marks lines for review rather than redacting them. Pair with a redaction recipe to act on the hits.

## Run

```
mog -m scan-for-pii <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
contact ada@example.com
just a normal line
ssn 123-45-6789 on file
```

Output:

```
contact ada@example.com # WARN(mog): possible PII (email)
just a normal line
ssn 123-45-6789 on file # WARN(mog): possible PII (ssn)
```

## Pipeline

- `detect_pii`: Tag lines that look like PII

## Tags

`scan` `pii` `audit`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

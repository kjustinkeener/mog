# Allowlist filter (intersect with a list)

Keep only lines that appear in an allowlist

Keep only the input lines that appear in an allowlist, dropping everything else. Point the 'allowlist' source at the approved set (permitted hosts, whitelisted emails, supported SKUs) with --source allowlist=<path>. Comparison trims each line and ignores case; input order and duplicates are preserved. This is a keep-what-is-approved filter; use 'new-lines-only' (subtract) for the opposite.

## Run

```
mog -m allowlist-filter <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
api.example.com
tracker.evil.example
CDN.example.com
  auth.example.com
metrics.thirdparty.net
api.example.com
mail.example.com
```

Output:

```
api.example.com
CDN.example.com
  auth.example.com
api.example.com
mail.example.com
```

## Steps

- `intersect`: Keep only lines whose text is on the allowlist

## Tags

`compare` `filter`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

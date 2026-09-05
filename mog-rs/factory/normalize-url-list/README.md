# Normalize and dedupe a URL list

Canonicalize, sort, and dedupe a URL list

Canonicalize each http(s) URL (lowercase scheme and host, drop the default port and a trailing host dot, sort query params), then sort and de-duplicate the list, so URLs that differ only cosmetically collapse to one entry. Non-URL lines pass through the normalize step and are sorted/deduped as-is.

## Run

```
mog -m normalize-url-list <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
HTTPS://Example.com:443/p?b=2&a=1
https://example.com/p?a=1&b=2
http://Host.:80/x
http://host/x
```

Output:

```
http://host/x
https://example.com/p?a=1&b=2
```

## Pipeline

- `normalize_url`: Lowercase host/scheme, drop default port, sort query
- `sort_lines`: Sort so duplicates are adjacent
- `remove_duplicate_lines`: Drop duplicate URLs

## Tags

`url` `normalize` `dedupe` `sort` `web`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

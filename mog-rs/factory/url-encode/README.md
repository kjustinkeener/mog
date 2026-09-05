# URL percent-encode

Percent-encode text for safe use in a URL

Percent-encode the text for safe use in a URL: the RFC 3986 unreserved set (letters, digits, - _ . ~) is left as-is and everything else becomes %XX. Useful for building query strings or encoding a value by hand. The inverse of url-decode.

## Run

```
mog -m url-encode <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
hello world & friends=100%
```

Output:

```
hello%20world%20%26%20friends%3D100%25
```

## Pipeline

- `url_encode`: Percent-encode the whole text

## Tags

`url` `encode` `escape` `single-action`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

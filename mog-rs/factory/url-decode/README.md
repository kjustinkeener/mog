# URL percent-decode

Reverse URL percent-encoding

Reverse percent-encoding, turning %XX sequences back into their characters (and + is left as-is; it is form-encoding, not RFC 3986). Errors if the result is not valid UTF-8. The inverse of url-encode.

## Run

```
mog -m url-decode <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
hello%20world%20%26%20friends%3D100%25
```

Output:

```
hello world & friends=100%
```

## Pipeline

- `url_decode`: Decode percent-encoding

## Tags

`url` `decode` `escape` `single-action`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

# Base64 Decode

Decode Base64 input back to text

Decode Base64 input back to text. Handy on Windows where a base64 CLI is not reliably present. Pairs with base64-encode.

## Run

```
mog -m base64-decode <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
SGVsbG8sIE1vZyEKTGluZSB0d28uCg==
```

Output:

```
Hello, Mog!
Line two.
```

## Pipeline

- `base64_decode`: Decode the Base64 text

## Tags

`base64` `decode`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

# Base64 Encode

Encode input as Base64

Encode the whole input as Base64. Handy on Windows where a base64 CLI is not reliably present. Pairs with base64-decode.

## Run

```
mog -m base64-encode <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
Hello, Mog!
Line two.
```

Output:

```
SGVsbG8sIE1vZyEKTGluZSB0d28uCg==
```

## Pipeline

- `base64_encode`: Encode the text as Base64

## Tags

`base64` `encode`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

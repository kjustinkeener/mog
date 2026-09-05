# Decode hex back to text

Unhexlify a hex string back to bytes/text

Decode each line of hex digits back to text (two hex digits per byte). The inverse of text-to-hex. Errors on an odd length or a non-hex character.

## Run

```
mog -m hex-to-text <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
48656c6c6f206d6f67
```

Output:

```
Hello mog
```

## Pipeline

- `hex_decode`: Decode each hex line back to text

## Tags

`decode` `hex` `encode` `single-action`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

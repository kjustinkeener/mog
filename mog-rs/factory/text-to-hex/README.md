# Encode text to hex

Hexlify: each byte becomes two lowercase hex digits

Hex-encode the input: every byte becomes two lowercase hex digits with no separator. The inverse of hex-to-text. Useful for inspecting or transmitting bytes as plain ASCII.

## Run

```
mog -m text-to-hex <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
Hello mog
```

Output:

```
48656c6c6f206d6f67
```

## Pipeline

- `hex_encode`: Hex-encode the input bytes

## Tags

`encode` `hex` `single-action`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

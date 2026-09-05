# Convert line endings to CRLF

Normalize all line endings to Windows CRLF

Convert every line ending to CRLF (Windows). The mirror of crlf-to-lf. Useful when a file must use DOS line endings (some Windows tools, certain network protocols).

## Run

```
mog -m lf-to-crlf <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
line one
line two
line three
```

Output:

```
line one
line two
line three
```

## Pipeline

- `eol_crlf`: Convert line endings to CRLF

## Tags

`eol` `normalize` `single-action`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

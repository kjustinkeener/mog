# CRLF to LF

Convert line endings to LF (dos2unix)

Convert all line endings to LF (Unix) -- the dos2unix transform. Just the EOL change, nothing else. Normalizes mixed or Windows CRLF (and old-Mac CR) files to LF.

## Run

```
mog -m crlf-to-lf <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
crlf one
crlf two
lf three
old-macfour
```

Output:

```
crlf one
crlf two
lf three
old-mac
four
```

## Pipeline

- `eol_lf`: Convert all line endings to LF

## Tags

`eol` `normalize`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

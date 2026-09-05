# Convert line endings to CR

Normalize all line endings to classic-Mac CR

Convert every line ending to a bare CR (classic Mac OS style). A niche companion to crlf-to-lf and lf-to-crlf for the rare tool or format that expects CR-only line breaks.

## Run

```
mog -m eol-to-cr-mac <file>
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
line oneline twoline three
```

## Pipeline

- `eol_cr`: Convert line endings to CR

## Tags

`eol` `normalize` `single-action`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

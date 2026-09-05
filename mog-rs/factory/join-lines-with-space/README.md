# Join lines with spaces

Replace line endings with single spaces

Replace every line ending with a single space, joining all lines into one without trimming the line contents. Similar to unwrap-to-one-line but it keeps each line's own leading/trailing whitespace.

## Run

```
mog -m join-lines-with-space <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
alpha
beta
gamma
```

Output:

```
alpha beta gamma 
```

## Pipeline

- `eol_to_space`: Replace line endings with spaces

## Tags

`whitespace` `join` `eol` `format` `single-action`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

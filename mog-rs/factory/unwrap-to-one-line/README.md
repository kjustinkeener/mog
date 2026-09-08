# Unwrap lines into one

Trim each line and join them with single spaces

Trim each line and join them all with single spaces, collapsing a hard-wrapped paragraph back into one line. Drops the internal line breaks. Useful for un-wrapping text copied from a fixed-width source.

## Run

```
mog -m unwrap-to-one-line <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
  This paragraph
  was hard wrapped
  across three lines.  
```

Output:

```
This paragraph was hard wrapped across three lines.
```

## Steps

- `trim_and_eol_to_space`: Trim each line, join with spaces

## Tags

`whitespace` `join` `format` `single-action`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

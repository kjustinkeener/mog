# Tidy List

Trim, dedupe, sort, and Title Case a messy list

Clean up a messy list: trim, drop blanks, de-duplicate, sort, Title Case.

## Run

```
mog -m tidy-list <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
  banana
Apple
cherry

apple
  BANANA
date
Cherry

elderberry
date
  fig
FIG
```

Output:

```
Apple
Banana
Cherry
Date
Elderberry
Fig
```

## Steps

- `trim_whitespace`: Trim spaces around each line
- `remove_empty_lines`: Drop blank lines
- `remove_duplicate_lines`: De-duplicate (case-insensitive)
- `sort_lines`: Sort A to Z (case-insensitive)
- `to_proper`: Title Case each word

## Tags

`text` `cleanup` `sort` `dedupe` `list`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

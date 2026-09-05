# Keep only the digits

Drop every non-digit character from each line

Keep only the digit characters on each line, dropping everything else (spaces, punctuation, letters). Line breaks are preserved. Handy for normalizing phone numbers, IDs, or codes to bare digits.

## Run

```
mog -m keep-digits-only <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
(555) 123-4567
ID: abc-123-xyz
Total 42 items
```

Output:

```
5551234567
123
42
```

## Pipeline

- `keep_chars`: Keep only digit characters

## Tags

`text` `numbers` `filter` `normalize` `contacts` `single-action`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

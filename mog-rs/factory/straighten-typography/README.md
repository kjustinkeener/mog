# Straighten smart typography to ASCII

Replace curly quotes, dashes, and ellipses with plain ASCII

Replace typographic punctuation with plain ASCII: curly quotes to straight ' and ", en/em dashes to hyphens, and the ellipsis character to three dots. Useful before diffing, grepping, or pasting text into code where smart punctuation causes trouble. Accented letters are left alone.

## Run

```
mog -m straighten-typography <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
She said “hello” and ‘hi’ — really… fine–ish.
```

Output:

```
She said "hello" and 'hi' - really... fine-ish.
```

## Steps

- `normalize`: Straighten quotes, dashes, and the ellipsis

## Tags

`text` `format` `quote` `normalize` `unicode`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

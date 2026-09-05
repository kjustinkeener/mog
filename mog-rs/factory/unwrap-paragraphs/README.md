# Unwrap hard-wrapped paragraphs

Join hard-wrapped lines back into flowing paragraphs

Join hard-wrapped lines back into flowing paragraphs: single line breaks within a paragraph become spaces, while blank-line paragraph breaks are preserved. The inverse of a fixed-width word wrap (and of one-sentence-per-line). Good for un-wrapping pasted email or text-file prose before reflowing or diffing.

## Run

```
mog -m unwrap-paragraphs <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
This is a paragraph
that was hard-wrapped
across three lines.

And here is a second
paragraph on two lines.
```

Output:

```
This is a paragraph that was hard-wrapped across three lines.

And here is a second paragraph on two lines.
```

## Pipeline

- `replace_regex`: Protect blank-line paragraph breaks
- `replace_regex`: Join the remaining line breaks with a space
- `replace`: Restore paragraph breaks
- `trim_whitespace_right`: Clean any trailing space

## Tags

`text` `prose` `format`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

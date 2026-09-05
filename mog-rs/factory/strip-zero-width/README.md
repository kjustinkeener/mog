# Strip invisible characters

Remove zero-width and invisible characters

Remove invisible characters that you cannot see to delete: zero-width space/joiner/non-joiner (U+200B/C/D), a byte-order mark or zero-width no-break space (U+FEFF), and the soft hyphen (U+00AD); a non-breaking space (U+00A0) is turned into a normal space. These often sneak in from copy-paste, PDFs, or web pages and break diffs, greps, and parsers.

## Run

```
mog -m strip-zero-width <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
he​llo world﻿ and soft­hyphen
```

Output:

```
hello world and softhyphen
```

## Pipeline

- `replace_regex`: Delete zero-width and BOM/soft-hyphen characters
- `replace_regex`: Turn a non-breaking space into a normal space

## Tags

`text` `unicode` `cleanup`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

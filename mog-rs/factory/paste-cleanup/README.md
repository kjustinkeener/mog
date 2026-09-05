# Paste Cleanup

Convert smart typography to plain ASCII

Normalize 'smart' typography from copied text to plain ASCII: curly double and single quotes to straight " and ', en dash to -, em dash to --, ellipsis to ..., non-breaking space to a normal space, and zero-width characters (ZWSP, ZWNJ, ZWJ, and a mid-text BOM) removed.

## Run

```
mog -m paste-cleanup <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
“Hello,” it’s a test…
range 3–4 and a non breaking space.
zero​width‌joined‍here﻿gone.
```

Output:

```
"Hello," it's a test...
range 3-4 and a non breaking space.
zerowidthjoinedheregone.
```

## Pipeline

- `replace_map`: Map smart typography to ASCII equivalents in a single pass

## Tags

`text` `cleanup` `unicode` `paste` `format`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

# Case-preserving find/replace

Rename a word while matching the surrounding case

Literal find/replace that casts the replacement to match the case of each occurrence: 'color' -> 'colour', 'Color' -> 'Colour', 'COLOR' -> 'COLOUR'. Handy for renaming an identifier or spelling variant without a separate pass per casing.

## Run

```
mog -m case-preserving-rename <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
color
Color
COLOR
the color red
```

Output:

```
colour
Colour
COLOUR
the colour red
```

## Steps

- `smart_case_replace`: Replace 'color' with 'colour', preserving case

## Tags

`case` `replace` `codemod` `single-action`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

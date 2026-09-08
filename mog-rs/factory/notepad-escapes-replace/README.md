# Literal replace with \t \n escapes

Find/replace that decodes Notepad++ escapes

Literal find/replace that first decodes Notepad++-style escapes ( \t \n \r ) in both the find and replacement. This mog turns tabs into a single space. Use it for escape-aware literal replacements without writing a regex.

## Run

```
mog -m notepad-escapes-replace <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
col1	col2	col3
a	b	c
```

Output:

```
col1 col2 col3
a b c
```

## Steps

- `replace_extended`: Replace each tab with a space

## Tags

`replace` `escape` `whitespace` `eol` `single-action`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

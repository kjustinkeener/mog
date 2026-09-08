# Wrap each line in double quotes

Wrap each line in double quotes

Surround each line with double quotes, turning a plain list into quoted strings; combine with join-lines-comma for an inline list. Does not escape quotes already inside a line; for safe JSON strings use lines-to-json-strings.

## Run

```
mog -m quote-lines <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
apple
banana
cherry
```

Output:

```
"apple"
"banana"
"cherry"
```

## Steps

- `wrap_lines`: Add a double quote before and after each line

## Tags

`text` `quote` `list` `format`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

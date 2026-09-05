# Reverse line order

Reverse the order of lines (like tac)

Reverse the order of lines (last line first), like tac. Useful for flipping a log so newest is on top, or reversing any list.

## Run

```
mog -m reverse-lines <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
first
second
third
```

Output:

```
third
second
first
```

## Pipeline

- `reverse_lines`: Reverse the line order

## Tags

`line` `sort` `single-action`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

# Join lines with commas

Join lines into a comma-separated list

Collapse a list (one item per line) into a single comma-and-space separated line. Blank lines are dropped first. Handy for turning a column into an inline list. Pair with quote-lines for a quoted list.

## Run

```
mog -m join-lines-comma <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
alpha
beta
gamma
```

Output:

```
alpha, beta, gamma
```

## Steps

- `remove_empty_lines`: Drop blank lines
- `join_lines`: Join with a comma and space

## Tags

`text` `join` `list` `csv` `format`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

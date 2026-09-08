# Split each line on commas

Explode a delimited line into one piece per line

Split each line on a literal separator (a comma here) and emit each piece as its own line, trimming surrounding whitespace. Turns a comma-joined line into a vertical list. Change the separator to split on any delimiter.

## Run

```
mog -m split-on-comma <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
red, green, blue
one,two,three
solo
```

Output:

```
red
green
blue
one
two
three
solo
```

## Steps

- `split_lines`: Split each line on commas, trimmed

## Tags

`line` `fragment` `csv` `single-action`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

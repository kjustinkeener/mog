# Shuffle lines (seeded)

Randomly reorder lines reproducibly

Randomly reorder the lines. A fixed seed makes the order reproducible, which keeps the golden stable and lets you re-run a shuffle deterministically. Useful for sampling or de-biasing an ordered list.

## Run

```
mog -m shuffle-lines <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
one
two
three
four
five
six
```

Output:

```
five
three
two
one
six
four
```

## Pipeline

- `shuffle_lines`: Reorder lines with a fixed seed

## Tags

`line` `sort` `random` `filter` `single-action`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

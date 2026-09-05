# Number a list

Turn a list into a numbered Markdown list

Turn a plain list (one item per line) into a numbered Markdown ordered list (1. first, 2. second, ...). Blank lines are dropped first.

## Run

```
mog -m number-list <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
Preheat the oven
Mix the batter
Bake for 30 minutes
```

Output:

```
1. Preheat the oven
2. Mix the batter
3. Bake for 30 minutes
```

## Pipeline

- `remove_empty_lines`: Drop blank lines
- `prefix_lines`: Prefix each line with a numbering marker
- `stamp_sequence`: Turn the markers into 1, 2, 3, ...

## Tags

`list` `markdown` `format`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

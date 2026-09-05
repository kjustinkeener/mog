# Zero-pad numbers

Left-pad numbers with leading zeros to a fixed width

Left-pad every run of digits to a fixed width with leading zeros so numeric IDs and filenames sort correctly (item-7 becomes item-0007). Width is set by the 'width' option (4 here). Runs already at or above the width are left unchanged. Only digit runs are touched; a leading sign stays outside the padding.

## Run

```
mog -m zero-pad-numbers <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
item-7
item-42
item-1000
item-12345
```

Output:

```
item-0007
item-0042
item-1000
item-12345
```

## Pipeline

- `pad_numbers`

## Tags

`numbers` `whitespace` `sort` `identifier` `single-action`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

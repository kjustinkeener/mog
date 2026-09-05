# Markdown Bulletize

Turn each line into a Markdown bullet

Turn each line into a Markdown bullet by prefixing '- '. For converting a plain list (one item per line) into a Markdown unordered list.

## Run

```
mog -m md-bulletize <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
apples
bananas
cherries
```

Output:

```
- apples
- bananas
- cherries
```

## Pipeline

- `prefix_lines`: Prefix each line with '- '

## Tags

`markdown` `list` `prefix`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

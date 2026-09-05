# Dedupe Keep Order

Remove duplicate lines, keep original order

Remove duplicate lines while preserving the original order (keep the first occurrence of each; case-sensitive). For files where line order is load-bearing and sort-dedupe would be wrong -- e.g. .gitignore, ordered config, a changelog.

## Run

```
mog -m dedupe-keep-order <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
banana
apple
banana
cherry
apple
date
```

Output:

```
banana
apple
cherry
date
```

## Pipeline

- `remove_duplicate_lines`: Keep the first occurrence, drop later duplicates

## Tags

`dedupe` `sort` `best-effort` `list` `config`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

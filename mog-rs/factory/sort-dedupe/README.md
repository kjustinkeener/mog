# Sort and Dedupe

Sort a list and remove duplicate lines

Normalize an order-independent list: trim each line, drop blank lines, remove duplicate lines (case-sensitive, first occurrence wins), then sort A to Z. For requirements.txt, wordlists, CODEOWNERS, allow/deny lists. NOT for files where line order is significant (e.g. .gitignore negation order) -- use dedupe-keep-order there.

## Run

```
mog -m sort-dedupe <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
banana
  apple  
Apple
banana
cherry

apple
   
Cherry
apple
```

Output:

```
Apple
Cherry
apple
banana
cherry
```

## Pipeline

- `trim_whitespace`: Trim leading/trailing whitespace
- `remove_empty_lines`: Drop blank lines
- `remove_duplicate_lines`: Remove duplicate lines (case-sensitive)
- `sort_lines`: Sort A to Z (case-sensitive)

## Tags

`sort` `dedupe` `list` `config` `ml` `cleanup` `normalize`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

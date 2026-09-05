# Add to every number

Increment each number by a constant

Add a constant (default 1) to every number in scope, preserving the surrounding text. Useful for bumping ports, ids, or version components across a list. Formatting of each number is preserved.

## Run

```
mog -m bump-numbers <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
port 8080
worker id 41
retry after 9s
```

Output:

```
port 8081
worker id 42
retry after 10s
```

## Pipeline

- `increment_numbers`: Add 1 to each number

## Tags

`text` `numbers` `math` `single-action`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

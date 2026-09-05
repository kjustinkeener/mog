# Diff a list against a baseline

Diff a list against a baseline, git-style

Show a line-by-line diff of the input against a baseline list, git-style: a leading '- ' marks a line only in the baseline (removed), '+ ' a line only in the input (added), and '  ' an unchanged line. Point the 'baseline' source at the old version with --source baseline=<path>. Good for a readable changelog between two snapshots (config keys, dependency lists, checklists). Set the 'format' option to only_added or only_removed to emit just one side, or unified for a unified diff.

## Run

```
mog -m list-diff <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
install dependencies
run migrations
warm cache
start server
```

Output:

```
  install dependencies
  run migrations
- seed database
+ warm cache
  start server
```

## Pipeline

- `diff`: Emit a git-style +/-/space marker diff of the input against the baseline

## Tags

`compare` `diff` `audit` `docs`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

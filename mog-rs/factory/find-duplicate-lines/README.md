# Keep only duplicated lines

Show the lines that occur more than once

Keep only the lines that occur more than once in the input (one copy of each), dropping the unique lines. The inverse of a de-dupe: use it to audit a list for repeats.

## Run

```
mog -m find-duplicate-lines <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
apple
banana
apple
cherry
banana
apple
```

Output:

```
apple
banana
```

## Pipeline

- `keep_duplicate_lines`: Keep only lines that occur more than once

## Tags

`line` `dedupe` `audit` `filter`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

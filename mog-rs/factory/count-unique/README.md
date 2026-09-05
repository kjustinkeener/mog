# Count unique lines

Collapse duplicate lines and count each (uniq -c)

Collapse duplicate lines and prefix each with how many times it occurred (like sort | uniq -c). Useful for tallying log messages, tags, or values.

## Run

```
mog -m count-unique <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
apple
banana
apple
cherry
apple
banana
```

Output:

```
3 apple
2 banana
1 cherry
```

## Pipeline

- `unique_with_count`: Collapse duplicates and prefix a count

## Tags

`line` `list` `dedupe` `single-action`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

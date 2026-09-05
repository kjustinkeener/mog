# Collapse adjacent duplicate lines

uniq: collapse runs of identical adjacent lines to one

Collapse runs of identical adjacent lines to a single line (the classic uniq behavior). Non-adjacent duplicates are left alone. Useful for squashing repeated log lines.

## Run

```
mog -m collapse-adjacent-dupes <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
start
start
run
run
run
start
done
```

Output:

```
start
run
start
done
```

## Pipeline

- `remove_consecutive_duplicate_lines`: Collapse adjacent duplicates

## Tags

`line` `dedupe` `whitespace` `single-action`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

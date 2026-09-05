# Lines to Markdown checklist

Turn a list into a Markdown checklist

Turn a plain list (one item per line) into a Markdown task list by prefixing each non-blank line with '- [ ] '. Blank lines are left as separators. Handy for turning notes or a pasted list into an actionable checklist.

## Run

```
mog -m lines-to-checklist <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
Write the spec
Review with the team

Ship it
```

Output:

```
- [ ] Write the spec
- [ ] Review with the team

- [ ] Ship it
```

## Pipeline

- `prefix_lines`: Prefix each non-blank line with an empty checkbox

## Tags

`markdown` `list` `docs`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

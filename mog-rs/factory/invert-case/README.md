# Invert letter case

Swap the case of every letter

Swap the case of every letter: uppercase becomes lowercase and vice versa. A mechanical, reversible transform (running it twice restores the original).

## Run

```
mog -m invert-case <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
Hello World
MixedCASE text
```

Output:

```
hELLO wORLD
mIXEDcase TEXT
```

## Pipeline

- `invert_case`: Swap the case of every letter

## Tags

`case` `text` `replace` `comment` `single-action`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

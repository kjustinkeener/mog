# Convert leading spaces to tabs

Convert leading spaces to tabs for indentation

Turn runs of leading spaces into tabs for indentation. Uses the spaces_to_tabs action's default tab width. The inverse of tabs-to-spaces.

## Run

```
mog -m spaces-to-tabs <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
def f():
    return 1
```

Output:

```
def f():
	return 1
```

## Steps

- `spaces_to_tabs`: Leading spaces -> tabs

## Tags

`whitespace` `indent` `single-action`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

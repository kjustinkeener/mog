# Tabs to Spaces

Expand tabs to spaces (default 4 per tab)

Expand tabs to spaces (default 4 per tab; override with -D width=N). Opt-in and deliberate: do NOT run on tab-significant files (Makefiles, Go source). Just the tab expansion, nothing else.

## Run

```
mog -m tabs-to-spaces <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
	indented
col1	col2	col3
no tabs here
```

Output:

```
    indented
col1    col2    col3
no tabs here
```

## Pipeline

- `tabs_to_spaces`: Replace each tab with spaces

## Tags

`whitespace` `indent` `format`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

# Strip Trailing Whitespace

Remove trailing spaces and tabs from every line

Remove trailing spaces and tabs from the end of every line. The minimal, precise transform: unlike repo-tidy it does NOT touch line endings, blank lines, or the final newline. Runs across many files at once with preview and backup.

## Run

```
mog -m strip-trailing-whitespace <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
no trailing here
trailing spaces   
trailing tab	
internal  spaces  kept   
   
after blank
```

Output:

```
no trailing here
trailing spaces
trailing tab
internal  spaces  kept

after blank
```

## Steps

- `trim_whitespace_right`: Remove trailing whitespace from each line

## Tags

`whitespace` `strip` `cleanup` `scan`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

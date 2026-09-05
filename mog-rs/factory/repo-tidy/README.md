# Repo Tidy

Normalize line endings, trailing space, and blank lines

Whitespace/EOL hygiene for source files: normalize CRLF to LF, strip trailing whitespace, collapse runs of blank lines, and ensure exactly one final newline. Does NOT touch tabs or internal spacing, so it is safe for tab-indented files and Makefiles. Built to run across many files at once with preview and backup.

## Run

```
mog -m repo-tidy <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
line one trailing ws   
	tab indent kept
internal  double  spaces  kept

   

after blank run
no final newline
```

Output:

```
line one trailing ws
	tab indent kept
internal  double  spaces  kept

after blank run
no final newline
```

## Pipeline

- `eol_lf`: Normalize line endings to LF
- `trim_whitespace_right`: Strip trailing whitespace from every line
- `squeeze_blank_lines`: Collapse runs of blank lines to a single blank line
- `replace_regex`: Ensure exactly one trailing newline

## Tags

`whitespace` `cleanup` `eol` `normalize`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

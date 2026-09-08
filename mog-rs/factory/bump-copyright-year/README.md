# Bump copyright year

Update copyright lines to the current year

Update the year in copyright lines to the current year ({{@year}}). Matches 'Copyright YYYY' and 'Copyright (c) YYYY-YYYY' (updating the last year). Pin a specific year with --pin-now.

## Run

```
mog -m bump-copyright-year <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
Copyright 2019 Acme Corp
Copyright (c) 2015-2018 Bob
No year on this line
```

Output:

```
Copyright 2006 Acme Corp
Copyright (c) 2015-2006 Bob
No year on this line
```

## Steps

- `replace_regex`: Replace the trailing year after Copyright with the current year.

## Tags

`codemod` `docs` `date` `config`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

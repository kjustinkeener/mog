# SQL finalize whitespace

Tidy whitespace in a generated SQL script

Canonical whitespace finalize for a generated SQL script: collapse the double spaces left by dropped markers, tidy stray spaces before punctuation (set tidy_chars to ';' or ',;'), strip trailing whitespace, and collapse runs of blank lines. Composable fragment: run last (via run_mog) from any converter that targets PostgreSQL. Assumes LF text.

## Run

```
mog -m sql-finalize-whitespace <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
CREATE TABLE public.orders (
  id          integer ,
  customer    text ;


  total       numeric
) ;
```

Output:

```
CREATE TABLE public.orders (
  id integer,
  customer text;

  total numeric
);
```

## Steps

- `squeeze_spaces`: Collapse the double spaces left by removing column/type markers (keeps indentation).
- `replace_regex`: Tidy any space left before punctuation (tidy_chars selects which characters).
- `trim_whitespace_right`: Remove trailing whitespace on every line.
- `squeeze_blank_lines`: Collapse runs of blank lines to a single blank line.

## Tags

`fragment` `sql` `postgres` `format` `whitespace`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

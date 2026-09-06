# git log to CSV

Convert git log output to a CSV commit table

Turn `git log` output into a CSV commit table (one row per commit) with the columns commit,author_name,author_email,author_date,subject. INPUT CONTRACT: git log has no canonical text shape, so this mog consumes exactly one format -- run `git log --pretty=format:'%H%x1f%an%x1f%ae%x1f%aI%x1f%s'` and pipe that in. %x1f writes an ASCII unit separator (U+001F) between fields, which cannot occur in a commit subject, so subjects holding commas, quotes or pipes stay intact. Fields are then re-joined as RFC 4180 CSV: a field is quoted only when it contains a comma, a quote or a line break, and internal quotes are doubled. Reorder or drop columns by editing the --pretty format and the header line on the prepend step. Lines that do not carry the expected five fields are flagged in place rather than dropped.

## Run

```
mog -m git-log-to-csv <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
9f3c1a2b7d4e5f60718293a4b5c6d7e8f9012345Ada Lovelaceada@example.com2026-08-14T09:12:03+00:00Fix parser: handle "quoted" args, commas, and trailing pipes
1b2c3d4e5f60718293a4b5c6d7e8f90123456789Grace Hoppergrace@example.com2026-08-13T17:45:11+00:00Add CSV export, TSV export, and a --delimiter flag
a0b1c2d3e4f50617283940a1b2c3d4e5f6071829Ken Thompsonken@example.com2026-08-12T08:02:59+00:00Bump version to 0.4.1
7c6b5a4938271605f4e3d2c1b0a99887766554433Barbara Liskovbarbara@example.com2026-08-11T22:31:40+00:00Revert "Cache the resolver" (it broke Windows paths)
```

Output:

```
commit,author_name,author_email,author_date,subject
9f3c1a2b7d4e5f60718293a4b5c6d7e8f9012345,Ada Lovelace,ada@example.com,2026-08-14T09:12:03+00:00,"Fix parser: handle ""quoted"" args, commas, and trailing pipes"
1b2c3d4e5f60718293a4b5c6d7e8f90123456789,Grace Hopper,grace@example.com,2026-08-13T17:45:11+00:00,"Add CSV export, TSV export, and a --delimiter flag"
a0b1c2d3e4f50617283940a1b2c3d4e5f6071829,Ken Thompson,ken@example.com,2026-08-12T08:02:59+00:00,Bump version to 0.4.1
7c6b5a4938271605f4e3d2c1b0a99887766554433,Barbara Liskov,barbara@example.com,2026-08-11T22:31:40+00:00,"Revert ""Cache the resolver"" (it broke Windows paths)"
```

## Pipeline

- `remove_empty_lines`: Drop blank lines (a trailing newline from git log leaves one).
- `flag_matching`: Flag any line that is not five unit-separated fields.
- `replace`: Double every embedded double quote.
- `replace_regex`: Wrap any field holding a comma or a quote in double quotes.
- `replace`: Swap each unit separator for a comma.
- `prepend`: Add the CSV header row.

## Tags

`git` `csv` `convert` `data` `audit`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

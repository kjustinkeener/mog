# Strip SQL comments

Remove SQL block and line comments

Remove SQL comments: /* block comments */ (including multi-line) and -- line comments (from the marker to end of line, with the preceding spaces/tabs), then trim any trailing whitespace left behind. Mechanical: a -- or /* that appears inside a string literal would also be stripped, so review queries with such literals. Assumes LF text.

## Run

```
mog -m strip-sql-comments <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
SELECT id, -- the id
  name /* full name */
FROM users; -- the table
```

Output:

```
SELECT id,
  name
FROM users;
```

## Steps

- `replace_regex`: Remove /* ... */ block comments (dotall, non-greedy)
- `replace_regex`: Remove -- line comments to end of line (with leading spaces/tabs)
- `trim_whitespace_right`: Clean up any trailing whitespace the removals left

## Tags

`sql` `comment` `strip` `cleanup` `minify`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

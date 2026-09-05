# Strip JSONC comments to plain JSON

Convert JSONC to plain JSON by removing comments

Remove // line comments and /* block */ comments from JSONC (JSON-with-comments, as used by tsconfig.json / VS Code settings) so it parses as strict JSON. Line comments are only stripped when the // is at the start of a line or preceded by whitespace, so a URL like http://example.com is left intact; trailing commas are NOT handled (see remove-trailing-commas). Mechanical: a // or /* inside a string value could still be affected.

## Run

```
mog -m strip-jsonc-comments <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
{
  // top-level comment
  "name": "demo", // inline comment
  "url": "http://example.com/path",
  /* a block
     comment */
  "port": 8080
}
```

Output:

```
{
  "name": "demo",
  "url": "http://example.com/path",
  "port": 8080
}
```

## Pipeline

- `replace_regex`: Remove /* block */ comments
- `replace_regex_multiline`: Remove full-line // comments
- `replace_regex`: Remove inline // comments (after whitespace, so URLs survive)
- `trim_whitespace_right`: Clean trailing whitespace left by removed comments
- `remove_empty_lines`: Drop the now-empty comment lines

## Tags

`json` `comment` `strip` `cleanup`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

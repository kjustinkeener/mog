# Paste cleanup: web article

Tidy text copied from a web article

Tidy text copied from a web article or HTML page. Trims trailing spaces, drops common boilerplate lines (Advertisement, Share this, Sign up / Subscribe, cookie-banner lines), removes inline [edit] and [12] footnote/citation markers, and collapses runs of blank lines to a single blank line. The boilerplate list is a fixed set of common phrases (case-insensitive), so site-specific chrome may still slip through.

## Run

```
mog -m paste-cleanup-web <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
The History of Widgets [edit]   



Widgets were invented in 1850 [1] by Smith.
Advertisement
They became popular quickly.   


Share this article
We use cookies to improve your experience.
The end.
```

Output:

```
The History of Widgets

Widgets were invented in 1850  by Smith.
They became popular quickly.

The end.
```

## Steps

- `remove_lines_matching`: Remove common ad / share / signup / cookie-banner lines (case-insensitive).
- `replace_regex_multiline`: Remove [edit] and [12] citation/footnote markers left inline.
- `trim_whitespace_right`: Remove trailing whitespace from each line (also cleans spaces left by marker removal).
- `squeeze_blank_lines`: Collapse runs of blank lines to a single blank line.

## Tags

`paste` `cleanup` `web` `prose` `html` `codegen`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

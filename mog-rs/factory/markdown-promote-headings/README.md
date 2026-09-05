# Promote Markdown headings one level

Promote Markdown headings one level (H2 to H1)

Remove one # from every ATX heading, so H2 becomes H1, H3 becomes H2, and so on (H1 is left as-is, since it is already the top level). The inverse of markdown-demote-headings. Only lines starting with two or more # are affected.

## Run

```
mog -m markdown-promote-headings <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
# Title
## Section
### Subsection
Body text.
```

Output:

```
# Title
# Section
## Subsection
Body text.
```

## Pipeline

- `replace_regex_multiline`: Drop one # from heading lines (H2 and deeper)

## Tags

`markdown` `docs` `convert`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

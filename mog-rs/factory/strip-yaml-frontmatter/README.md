# Strip YAML frontmatter

Remove leading YAML frontmatter from a Markdown file

Remove a leading YAML frontmatter block (the metadata fenced by --- at the very top of a Markdown file), leaving the body content. Only a block at the start of the file is removed; later --- horizontal rules are untouched. Assumes LF text.

## Run

```
mog -m strip-yaml-frontmatter <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
---
title: Hello World
tags: [a, b]
date: 2024-01-01
---
# Hello World

Body content here.

---

A horizontal rule above stays.
```

Output:

```
# Hello World

Body content here.

---

A horizontal rule above stays.
```

## Pipeline

- `replace_regex`: Delete the opening ---...--- block at the start of the file

## Tags

`markdown` `docs` `yaml` `strip`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

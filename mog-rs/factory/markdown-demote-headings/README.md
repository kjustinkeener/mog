# Demote Markdown headings one level

Demote Markdown headings one level (H1 to H2)

Add one # to every ATX heading, so H1 becomes H2, H2 becomes H3, and so on (H6 is left as-is, since it is the deepest level Markdown allows). Handy when nesting a whole document beneath a new top-level heading. Only lines that start with # are affected; code fences and text are left alone.

## Run

```
mog -m markdown-demote-headings <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
# Title
Some text.
## Section
### Subsection
###### Deep
```

Output:

```
## Title
Some text.
### Section
#### Subsection
###### Deep
```

## Pipeline

- `replace_regex_multiline`: Prepend a # to heading lines up to H5

## Tags

`markdown` `docs` `convert`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

# Generate a Markdown table of contents

Build a linked table of contents from Markdown headings

Build a nested, linked table of contents from a Markdown file's ATX headings: one bullet per heading, indented by heading depth, linking to a GitHub-style slug anchor - [Title](#title). The visible title keeps its exact case and punctuation while the anchor is slugified to match GitHub's convention. Emits only the TOC; paste it where you want it. GitHub parity is approximate: duplicate headings do not get -1/-2 suffixes, an underscore becomes a hyphen, and a heading containing a tab is not supported.

## Run

```
mog -m markdown-toc <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
# Getting Started
Intro paragraph.

## Installation
### From Source
## Q&A: What's New? (v2)
Some text.

# API Reference
## Endpoints & Errors
```

Output:

```
- [Getting Started](#getting-started)
  - [Installation](#installation)
    - [From Source](#from-source)
  - [Q&A: What's New? (v2)](#qa-whats-new-v2)
- [API Reference](#api-reference)
  - [Endpoints & Errors](#endpoints-errors)
```

## Pipeline

- `keep_lines_matching`: Keep only ATX heading lines
- `replace_regex_multiline`: Duplicate each title, tab-separated (anchor copy on the right)
- `remove_chars`: Delete punctuation (except - and _) from the anchor field, so slugify does not hyphenate it -- matches GitHub, which strips punctuation
- `slugify`: Slugify only the second (anchor) field; the title field keeps its case and punctuation
- `replace_regex_multiline`: H1 -> top-level bullet
- `replace_regex_multiline`: H2
- `replace_regex_multiline`: H3
- `replace_regex_multiline`: H4
- `replace_regex_multiline`: H5
- `replace_regex_multiline`: H6

## Tags

`markdown` `docs` `links`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

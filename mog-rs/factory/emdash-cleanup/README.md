# Em Dash Cleanup

Replace em dashes with plain ASCII punctuation

Replace em dashes (U+2014), horizontal bars (U+2015), and runs of them with plain ASCII punctuation, preserving spacing: a dash between words becomes a spaced hyphen ' - ', a dash opening a line becomes '- ', and a trailing dash at end of line is dropped along with the space before it. Use it to strip em dashes from prose, commit messages, and docs.

## Run

```
mog -m emdash-cleanup <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
Mog is deterministic―it applies the same edit everywhere.
The engine ― written in Rust ― is fast.
― an opening dash line
dangling thought ―
long run a ―― b
no dashes here
```

Output:

```
Mog is deterministic - it applies the same edit everywhere.
The engine - written in Rust - is fast.
- an opening dash line
dangling thought
long run a - b
no dashes here
```

## Pipeline

- `replace_regex`: Line-opening dash becomes a plain hyphen bullet
- `replace_regex`: Trailing dash at end of line is dropped with its leading space
- `replace_regex`: Mid-line dash becomes a spaced hyphen

## Tags

`text` `cleanup` `unicode` `prose` `punctuation`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

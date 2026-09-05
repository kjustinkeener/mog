# Em Dash Audit

Flag each dash for review instead of rewriting it

Report every dash that needs a human or model decision instead of changing it: each line holding an em dash, horizontal bar, en dash, or two-em dash gets a review marker naming the choice (comma, colon, parentheses, or a new sentence), and dashes that are purely mechanical (a line-opening dash or one trailing at end of line) are marked as handled by dash-cleanup. Preview with --diff or gate with --check; pair it with dash-cleanup, which does the mechanical rewrite.

## Run

```
mog -m emdash-audit <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
The engine ― written in Rust ― is fast.
― an opening dash line
dangling thought ―
range 3–4 stays tight.
no dashes here
```

Output:

```
The engine ― written in Rust ― is fast. # FIXME(mog): dash needs a call: comma, colon, parentheses, or a new sentence
― an opening dash line # NOTE(mog): mechanical dash: dash-cleanup rewrites this
dangling thought ― # NOTE(mog): mechanical dash: dash-cleanup rewrites this
range 3–4 stays tight. # NOTE(mog): range dash: dash-cleanup makes this a tight hyphen
no dashes here
```

## Pipeline

- `flag_matching`: Mid-sentence dash: needs a judgment call
- `flag_matching`: En dash between word characters is a range: mechanical
- `flag_matching`: Line-opening or trailing dash: mechanical

## Tags

`text` `review` `unicode` `prose` `audit`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

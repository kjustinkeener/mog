# Em Dash Review List

List every dash line with its number and a verdict

Produce a compact review list: every line containing a dash, with its original line number and a marker saying whether the dash needs a judgment call (comma, colon, parentheses, or a new sentence) or is mechanical (a range, or a line-opening or trailing dash that dash-cleanup rewrites). Every other line is dropped, so the output is the review queue itself; raise the last step's context option to keep surrounding lines when the file is code rather than prose. Read it, decide each flagged line, then edit the source; it is a report, not a rewrite of your file.

## Run

```
mog -m emdash-review-list <file>
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
1. The engine ― written in Rust ― is fast. # FIXME(mog): dash needs a call: comma, colon, parentheses, or a new sentence
2. ― an opening dash line # NOTE(mog): mechanical dash: dash-cleanup rewrites this
3. dangling thought ― # NOTE(mog): mechanical dash: dash-cleanup rewrites this
4. range 3–4 stays tight. # NOTE(mog): range dash: dash-cleanup makes this a tight hyphen
```

## Pipeline

- `flag_matching`: Mid-sentence dash: needs a judgment call
- `flag_matching`: En dash between word characters is a range: mechanical
- `flag_matching`: Line-opening or trailing dash: mechanical
- `number_lines`: Keep the original line number with each line
- `keep_lines_matching`: Drop every line that carries no marker (raise context to keep surrounding lines)

## Tags

`text` `review` `unicode` `prose` `audit`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

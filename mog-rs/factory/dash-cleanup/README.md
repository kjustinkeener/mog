# Dash Cleanup

Normalize every Unicode dash to ASCII hyphens

Normalize the whole Unicode dash family to ASCII: em dash and horizontal bar (via emdash-cleanup) plus two-em and three-em dashes become a spaced hyphen, an en dash between word characters becomes a tight hyphen so ranges like 3-4 stay tight, a standalone or spaced en dash becomes a spaced hyphen, figure dash, non-breaking hyphen and the minus sign become a plain hyphen, and soft hyphens are removed. Use it on prose, docs, and commit messages that came from a word processor or a chat model.

## Run

```
mog -m dash-cleanup <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
range 3–4 and pages 10–12.
The engine – written in Rust – is fast.
non‑breaking and soft­hyphen words.
temperature −5 degrees.
wide gap a ⸺ b
```

Output:

```
range 3-4 and pages 10-12.
The engine - written in Rust - is fast.
non-breaking and softhyphen words.
temperature -5 degrees.
wide gap a - b
```

## Pipeline

- `replace_regex`: En dash inside a word or number is a range: make it a tight hyphen
- `replace_map`: Fold the remaining dash variants onto forms emdash-cleanup handles
- `replace_regex`: Line-opening dash becomes a plain hyphen bullet
- `replace_regex`: Trailing dash at end of line is dropped with its leading space
- `replace_regex`: Mid-line dash becomes a spaced hyphen

## Tags

`text` `cleanup` `unicode` `prose` `punctuation`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

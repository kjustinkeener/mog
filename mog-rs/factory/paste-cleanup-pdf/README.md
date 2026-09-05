# Paste cleanup: PDF copy

Repair line-break artifacts from PDF-copied text

Repair common line-break artifacts of text copied from a PDF. Removes lone page-number lines (a bare number, or 'Page 12'), re-joins words hyphenated across a line break (exam-\nple -> example), and reflows lines wrapped mid-sentence (a line ending in a lowercase letter or comma followed by a line starting lowercase is joined with a space). Not a layout parser: it does not detect or strip repeated running headers/footers, and an aggressive reflow can join two lines meant to stay apart.

## Run

```
mog -m paste-cleanup-pdf <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
The quick brown fox jumped over the lazy
dog in the meadow. It was a sunny exam-
ple of a fine day.

12

A new paragraph starts here. The end
of the story was near.
Page 13
The End.
```

Output:

```
The quick brown fox jumped over the lazy dog in the meadow. It was a sunny example of a fine day.

A new paragraph starts here. The end of the story was near.
The End.
```

## Pipeline

- `remove_lines_matching`: Remove lines that are only a page number (optionally 'Page N').
- `replace_regex`: Join a word split as prefix-<break>suffix across a line break.
- `replace_regex`: Join a line ending in a lowercase letter or comma to a next line starting lowercase, with a space.
- `squeeze_blank_lines`: Collapse the blank runs left where page-number lines were removed to a single blank line.

## Tags

`paste` `cleanup` `convert` `format` `text`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

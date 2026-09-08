# Wrap text as a Markdown blockquote

Wrap text as a Markdown blockquote

Prefix every line with '> ' so a block of text becomes a Markdown blockquote, for quoting an email, log excerpt, or passage inside a Markdown document. Every line including blanks is prefixed, keeping the quote contiguous.

## Run

```
mog -m quote-as-blockquote <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
First line of the quote.
Second line of the quote.
```

Output:

```
> First line of the quote.
> Second line of the quote.
```

## Steps

- `prefix_lines`: Prefix each line with a blockquote marker

## Tags

`markdown` `quote` `docs` `format`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

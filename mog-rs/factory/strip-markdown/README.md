# Strip Markdown

Reduce Markdown to plain text

Reduce Markdown to plain text: drop code-fence lines, unwrap links and images to their text, and remove header, blockquote, and list markers, horizontal rules, and bold/italic/inline-code/strikethrough emphasis. Underscore emphasis is left as-is so snake_case is not corrupted. Not a Markdown parser.

## Run

```
mog -m strip-markdown <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
# Project Notes

Some **bold** and *italic* and `inline code` in a line.

- first item
- second item

> a quoted aside

See [the docs](https://example.com/guide) for details.
```

Output:

```
Project Notes

Some bold and italic and inline code in a line.

first item
second item
a quoted aside

See the docs for details.
```

## Pipeline

- `strip_markdown`: Strip the Markdown syntax.

## Tags

`markdown` `text` `cleanup` `strip` `plaintext`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

# Strip HTML comments

Remove HTML and Markdown comments, including multi-line

Remove HTML/Markdown comments (<!-- ... -->), including ones that span multiple lines. Useful for cleaning authoring notes out of HTML or Markdown before publishing. The comment markers and everything between them are deleted; surrounding text is left as-is. Assumes LF text.

## Run

```
mog -m strip-html-comments <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
<h1>Title</h1>
<!-- a note to self -->
<p>Visible.</p>
<!-- multi
line
comment -->
<p>Also visible.</p>
```

Output:

```
<h1>Title</h1>

<p>Visible.</p>

<p>Also visible.</p>
```

## Pipeline

- `replace_regex`: Delete <!-- ... --> comment spans (dotall, non-greedy)

## Tags

`html` `markdown` `comment` `strip` `cleanup`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

# Strip HTML to plain text

Remove HTML tags and decode entities

Convert a snippet of HTML to plain text in two steps: remove every <...> tag, then decode HTML entities ( &amp; &lt; &#39; etc. ) back to their characters. A de-markup pass, not a full HTML parser.

## Run

```
mog -m html-to-text <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
<p>Hello &amp; welcome</p>
<b>Bold</b> &lt;tag&gt;
```

Output:

```
Hello & welcome
Bold <tag>
```

## Steps

- `strip_html_tags`: Remove HTML/XML tags
- `html_decode`: Decode HTML entities to characters

## Tags

`html` `text` `strip` `markdown` `convert`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

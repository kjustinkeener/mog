# Decode HTML entities

Decode HTML entities to characters

Reverse HTML entity encoding: named entities (&amp; &lt; &copy; and many more), decimal numeric entities (&#8364;), and hex numeric entities (&#x1F600;) all become their characters. An unknown &name; is left as-is. Useful for turning scraped or exported HTML back into plain text. The inverse of html-escape.

## Run

```
mog -m html-unescape <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
Tom &amp; Jerry say &quot;hi&quot; for &#8364;5 &copy;2024
```

Output:

```
Tom & Jerry say "hi" for €5 ©2024
```

## Pipeline

- `html_decode`: Decode HTML entities to characters

## Tags

`html` `escape` `decode` `text` `single-action`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

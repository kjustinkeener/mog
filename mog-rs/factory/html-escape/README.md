# HTML Escape

Escape HTML special characters

Escape HTML special characters (& < > and quotes) so text or code can be embedded safely in HTML or Markdown. Runs over the whole input.

## Run

```
mog -m html-escape <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
if (a < b && c > d) return "x";
Tom & Jerry <tag>
```

Output:

```
if (a &lt; b &amp;&amp; c &gt; d) return &quot;x&quot;;
Tom &amp; Jerry &lt;tag&gt;
```

## Pipeline

- `html_encode`: Escape HTML special characters

## Tags

`html` `escape` `encode` `markdown` `redact`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

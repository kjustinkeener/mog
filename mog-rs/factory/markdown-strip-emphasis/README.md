# Strip Markdown bold/italic emphasis

Remove Markdown bold and italic asterisk markers

Remove asterisk emphasis markers, keeping the text: **bold** becomes bold and *italic* becomes italic. Underscore emphasis (_x_) is left alone to protect snake_case identifiers. For a fuller Markdown-to-text pass, see strip-markdown.

## Run

```
mog -m markdown-strip-emphasis <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
This is **bold** and *italic* and a snake_case name.
Use **strong** words *sparingly*.
```

Output:

```
This is bold and italic and a snake_case name.
Use strong words sparingly.
```

## Steps

- `replace_regex`: **bold** -> bold
- `replace_regex`: *italic* -> italic

## Tags

`markdown` `strip` `text`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

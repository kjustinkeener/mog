# Markdown to Slack mrkdwn

Convert Markdown to Slack mrkdwn

Convert common Markdown to Slack's mrkdwn so a note pastes cleanly into Slack: [text](url) becomes <url|text>, an ATX heading becomes *bold* text, and **bold** becomes Slack's single-asterisk *bold*. Italics and lists are left as-is (Slack renders * bullets and _italic_ differently, so those are safer untouched).

## Run

```
mog -m markdown-to-slack <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
# Release Notes
This ships **bold** fixes. See the [changelog](https://example.com/changes).
```

Output:

```
*Release Notes*
This ships *bold* fixes. See the <https://example.com/changes|changelog>.
```

## Pipeline

- `replace_regex`: Links [text](url) -> <url|text>
- `replace_regex_multiline`: Headings -> *bold*
- `replace_regex`: **bold** -> *bold*

## Tags

`markdown` `chat` `convert`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

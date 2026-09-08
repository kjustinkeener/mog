# Wrap text in a Markdown code block

Wrap the whole input in a Markdown code block

Fence the whole input in a triple-backtick Markdown code block, so a command's output, a log, or a snippet can be pasted into Markdown as preformatted text. Edit the opening fence on the step to add a language (e.g. ```json) for syntax highlighting.

## Run

```
mog -m wrap-in-code-block <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
def hello():
    print("hi")
```

Output:

````
```
def hello():
    print("hi")
```
````

## Steps

- `prepend`: Opening fence
- `append`: Closing fence

## Tags

`markdown` `codemod` `format` `docs`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

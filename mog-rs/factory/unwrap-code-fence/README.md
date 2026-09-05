# Unwrap a fenced code block

Remove triple-backtick fence lines, keeping the code

Remove the triple-backtick fence lines (opening ```lang and closing ```) from Markdown, leaving just the code inside. Useful for extracting the raw content of a fenced block. Every fence line is dropped, so on a document with multiple code blocks the prose between them remains too.

## Run

```
mog -m unwrap-code-fence <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

````
```python
def hello():
    print("hi")
```
````

Output:

```
def hello():
    print("hi")
```

## Pipeline

- `remove_lines_matching`: Drop lines that are a ``` fence

## Tags

`markdown` `codemod` `strip` `extract`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

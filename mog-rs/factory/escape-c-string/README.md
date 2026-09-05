# Escape as a C/Java string literal

Backslash-escape each line and wrap it in quotes

Escape each line as a C / C++ / Java string literal: backslashes and double quotes are backslash-escaped and the whole line is wrapped in double quotes. Use to paste literal text into source code.

## Run

```
mog -m escape-c-string <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
say "hi"
path\to\file
plain text
```

Output:

```
"say \"hi\""
"path\\to\\file"
"plain text"
```

## Pipeline

- `escape_c`: Escape and quote each line as a C string literal

## Tags

`escape` `codemod` `java` `encode` `string` `single-action`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

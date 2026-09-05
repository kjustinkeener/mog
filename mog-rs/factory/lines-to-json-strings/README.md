# Wrap each line as a JSON string

Escape and quote each line as a JSON string

Escape and quote each input line as a JSON string literal, so arbitrary text (with quotes, backslashes, tabs, or control characters) can be safely embedded in JSON. Hand-escaping this is easy to get subtly wrong; this is exact. Set quote:false on the step to escape without the surrounding double quotes when you are building a larger string. Pair with join_lines to assemble a JSON array.

## Run

```
mog -m lines-to-json-strings <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
plain text
say "hello" to them
path C:\Users\me
```

Output:

```
"plain text"
"say \"hello\" to them"
"path C:\\Users\\me"
```

## Pipeline

- `escape_json`: Escape + quote each line as a JSON string

## Tags

`encode` `escape` `json` `string` `security`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

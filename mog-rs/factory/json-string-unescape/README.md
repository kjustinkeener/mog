# Decode JSON string escapes

Turn \n \t \uXXXX etc. into the real characters

Decode JSON string escape sequences ( \n \t \r \" \\ \uXXXX ) in the input into the characters they represent. Useful for un-wrapping a value that was logged or embedded as an escaped JSON string so it becomes readable again.

## Run

```
mog -m json-string-unescape <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
Line one\nLine two\tTabbed\nUnicode: \u0041\u0042
```

Output:

```
Line one
Line two	Tabbed
Unicode: AB
```

## Pipeline

- `json_unescape`: Decode JSON string escapes

## Tags

`json` `decode` `escape` `string` `single-action`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

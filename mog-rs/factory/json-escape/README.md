# JSON-escape text

Escape text into a JSON string literal

Escape arbitrary text into a valid JSON string literal: backslash and double-quote escaped, control characters escaped (\n \t \r \b \f, and \u00XX for the rest), and the whole wrapped in double quotes. Paste-ready for embedding a text blob (a log line, a message, multi-line content) into JSON or code. One pass, no chaining.

## Run

```
mog -m json-escape <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
He said "hi"	and a \ here
line two
```

Output:

```
"He said \"hi\"\tand a \\ here\nline two\n"
```

## Steps

- `replace_map`: Escape backslash, quote, and control characters in one pass
- `prepend`: Open the JSON string
- `append`: Close the JSON string

## Tags

`json` `escape` `encode` `string` `quote`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

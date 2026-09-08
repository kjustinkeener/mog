# Remove control characters

Delete non-printable control characters

Remove non-printable control characters, always keeping tab, newline, and carriage return. Useful for cleaning text pasted from a terminal or a binary-tainted file so it renders cleanly.

## Run

```
mog -m strip-control-chars <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
cleantext
bell and esc here
normal line
```

Output:

```
cleantext
bell and esc here
normal line
```

## Steps

- `strip_control_chars`: Delete control characters

## Tags

`text` `cleanup` `redact` `single-action`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

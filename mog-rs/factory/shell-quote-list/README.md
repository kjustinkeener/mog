# Shell-quote a list of arguments

Single-quote each line as a safe shell argument

Single-quote each line so it is safe to paste as one POSIX-shell word, even with spaces, quotes, or shell metacharacters. Embedded single quotes are handled with the standard close-quote/escaped-quote/reopen idiom. Hand-quoting shell arguments is easy to get dangerously wrong; this is exact.

## Run

```
mog -m shell-quote-list <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
plain
has spaces
it's tricky
; rm -rf /
```

Output:

```
'plain'
'has spaces'
'it'\''s tricky'
'; rm -rf /'
```

## Pipeline

- `escape_shell`: Wrap each line as one safe shell word

## Tags

`shell` `escape` `quote` `security`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

# Strip leading line numbers

Remove leading line numbers from pasted code

Remove leading line numbers from pasted code or output: a run of digits at the start of each line, with an optional : . or ) and the following whitespace, is deleted (so '  12: code' becomes 'code'). Indentation after the number is kept when it is the only separator. Lines that do not start with a number are left alone.

## Run

```
mog -m strip-line-numbers <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
  1: import os
 12: def main():
100  print("hi")
no number here
```

Output:

```
import os
def main():
print("hi")
no number here
```

## Pipeline

- `replace_regex_multiline`: Drop a leading number (and its separator) from each line

## Tags

`codemod` `cleanup` `line-numbers` `paste` `strip`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

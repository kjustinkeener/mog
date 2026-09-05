# Add line numbers

Prefix each line with a right-aligned line number

Prefix each line with its line number (right-aligned) and a separator, the way a code viewer shows them. Handy for referring to specific lines when sharing a file. To remove line numbers again, see strip-line-numbers.

## Run

```
mog -m add-line-numbers <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
import os
def main():
    print(os.getcwd())
```

Output:

```
1: import os
2: def main():
3:     print(os.getcwd())
```

## Pipeline

- `number_lines`: Prefix each line with its number

## Tags

`line-numbers` `codemod` `format` `text`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

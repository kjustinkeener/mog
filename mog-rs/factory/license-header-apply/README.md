# Apply an SPDX license header

Add an SPDX license header unless already present

Prepend an SPDX license header to a source file, but only if it is not already there -- so running it across a tree (or twice) never double-stamps. Idempotent: it checks for the SPDX-License-Identifier marker and inserts nothing when present. Edit the header text and marker on the step for your license and comment style.

## Run

```
mog -m license-header-apply <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
def main():
    print("hello")
```

Output:

```
# SPDX-License-Identifier: MIT
def main():
    print("hello")
```

## Steps

- `insert_if_absent`: Add the SPDX header unless it is already present

## Tags

`docs` `headers` `codemod` `normalize`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

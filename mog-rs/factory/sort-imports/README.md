# Sort import lines

Alphabetize each run of import statements

Sort each run of consecutive import statements alphabetically, in place, leaving blank lines and code as anchors so grouped import sections stay separate. The default pattern recognizes import / from / #include / using / use / require / @import across common languages; override 'pattern' to target one language precisely, and set 'dedupe' to drop duplicate imports.

## Run

```
mog -m sort-imports <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
import os
import collections
from pathlib import Path
import abc

import sys
print("hi")
```

Output:

```
from pathlib import Path
import abc
import collections
import os

import sys
print("hi")
```

## Pipeline

- `sort_imports`: Alphabetize each consecutive import block

## Tags

`imports` `sort` `codemod` `cleanup` `python` `javascript`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

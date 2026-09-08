# ES modules to CommonJS

Convert ES module syntax to CommonJS

Rewrite ES module syntax to CommonJS: 'import X from "m"' and 'import { a, b } from "m"' become const ... = require("m"), and 'export default X' becomes module.exports = X. The inverse of commonjs-to-esm. Does not handle namespace imports (* as), renamed exports, dynamic import(), or import side-effects.

## Run

```
mog -m esm-to-commonjs <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
import fs from 'fs';
import { join, resolve } from 'path';

export default myApp;
```

Output:

```
const fs = require('fs');
const { join, resolve } = require('path');

module.exports = myApp;
```

## Steps

- `replace_regex_multiline`: Default import -> const require
- `replace_regex_multiline`: Named import -> const destructure require
- `replace_regex_multiline`: Default export -> module.exports

## Tags

`javascript` `imports` `codemod` `convert`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

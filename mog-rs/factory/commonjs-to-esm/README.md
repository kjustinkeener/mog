# CommonJS to ESM

Convert CommonJS require/exports to ESM

Rewrite common CommonJS module syntax to ES modules: require destructuring and default imports, module.exports, and exports.NAME. Regex-based; unusual cases are left for manual review.

## Run

```
mog -m commonjs-to-esm <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
const fs = require('fs');
const { join } = require('path');
exports.run = function () {};
module.exports = main;
```

Output:

```
import fs from 'fs';
import { join } from 'path';
export const run = function () {};
export default main;
```

## Pipeline

- `replace_regex_multiline`: const { a } = require('x')  ->  import { a } from 'x'
- `replace_regex_multiline`: const x = require('x')  ->  import x from 'x'
- `replace_regex_multiline`: module.exports = X  ->  export default X
- `replace_regex_multiline`: exports.NAME =  ->  export const NAME =

## Tags

`codemod` `javascript` `imports` `config`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

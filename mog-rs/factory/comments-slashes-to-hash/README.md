# Convert // comments to # comments

Convert // line comments to # comments

Rewrite C/JS-style line comments (//) to hash comments (#) at the start of each comment line, preserving indentation and the comment text. Only lines whose first non-whitespace is // are touched, so code is left alone. Handy when porting a config or script between comment conventions; change from/to for other pairs (hash, slashes, dash, semicolon). Block comments (/* */) are not handled.

## Run

```
mog -m comments-slashes-to-hash <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
// database settings
host = localhost
port = 8080
  // indented note about the port
url = "http://example.com"
```

Output:

```
# database settings
host = localhost
port = 8080
  # indented note about the port
url = "http://example.com"
```

## Steps

- `convert_comment_style`: // -> # at the start of comment lines

## Tags

`text` `comment` `convert` `codemod` `config`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

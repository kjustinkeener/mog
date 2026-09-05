# Extract added lines from a git diff

Extract added lines from a git diff

Pull just the added lines out of a unified diff (git diff / patch), with the leading + removed, so you get the new content on its own. Drops the +++ file-header lines first, keeps only + lines, then strips the +. Useful for reviewing or re-using exactly what a patch adds. For removed lines, adapt the patterns to -.

## Run

```
mog -m git-diff-added-lines <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
diff --git a/f.txt b/f.txt
--- a/f.txt
+++ b/f.txt
@@ -1,3 +1,3 @@
 unchanged line
-removed line
+new line one
+new line two
```

Output:

```
new line one
new line two
```

## Pipeline

- `remove_lines_matching`: Drop the +++ file-header lines
- `keep_lines_matching`: Keep only added (+) lines
- `replace_regex_multiline`: Strip the leading +

## Tags

`config` `diff` `extract` `audit`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

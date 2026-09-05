# Escape regex metacharacters

Escape a line so it matches literally in a regex

Escape every regex metacharacter on each line so the text matches itself literally when used as a pattern. Use when you have a fixed string (a path, a version, an IP) that you need to embed safely into a larger regex.

## Run

```
mog -m regex-escape <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
a.b*c+d?
(group)[set]{2}
price $9.99
```

Output:

```
a\.b\*c\+d\?
\(group\)\[set\]\{2\}
price \$9\.99
```

## Pipeline

- `escape_regex`: Escape regex metacharacters

## Tags

`escape` `regex` `encode` `single-action`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

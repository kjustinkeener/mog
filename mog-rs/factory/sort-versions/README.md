# Sort version numbers

Sort version numbers by semver order

Sort a list of version strings like sort -V, with semver pre-release rules: 1.9.0 sorts before 1.10.0, and 1.0.0-rc.1 before the final 1.0.0. The version is the first token of each line (a leading 'v' is ignored); non-version lines keep their order at the end.

## Run

```
mog -m sort-versions <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
v1.10.0
1.2.0
1.9.0
1.10.0-rc.2
1.10.0-rc.10
1.2.0
latest
```

Output:

```
1.2.0
1.2.0
1.9.0
1.10.0-rc.2
1.10.0-rc.10
v1.10.0
latest
```

## Steps

- `sort_versions`: Order lines by version value

## Tags

`sort` `version` `list`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

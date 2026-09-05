# Assign a UUID to each line

Prefix each line with a fresh UUID

Prefix every non-empty line with a fresh RFC-4122 v4 UUID and a comma, using the per-match $uuid replacement token (one distinct UUID per line). A live run produces random UUIDs; runs are reproducible only when the RNG is pinned (--pin-seed, and under mog --test). Handy for tagging records or log lines with a within-run unique id.

## Run

```
mog -m assign-uuid-per-line <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
alpha
bravo
charlie
```

Output:

```
7f6f2ccd-b23f-4abb-bb69-278e947c01c6,alpha
160a31cf-02c1-4d06-90f6-e5ab1d768b95,bravo
117be1de-549d-4d43-a2c4-711f11efa0c5,charlie
```

## Pipeline

- `replace_regex`

## Tags

`identifier` `codegen` `comment` `single-action`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

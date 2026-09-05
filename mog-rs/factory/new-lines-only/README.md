# New lines only (subtract a baseline)

Keep only lines not in a baseline list

Keep only the input lines that are NOT already in a baseline list, so you see just what is new. Point the 'baseline' source at the previous snapshot (a known-good allowlist, yesterday's dependency list, an already-triaged error set) with --source baseline=<path>; each surviving line is one the baseline did not contain. Lines are trimmed before comparing, and input order and duplicates are preserved.

## Run

```
mog -m new-lines-only <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
bravo
foxtrot
  charlie
golf
alpha
foxtrot
hotel
```

Output:

```
foxtrot
golf
foxtrot
hotel
```

## Pipeline

- `subtract`: Drop every line present in the baseline list; keep the rest

## Tags

`compare` `diff` `audit` `dedupe` `filter`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

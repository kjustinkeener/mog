# Fill placeholders from a list

Replace each marker with the next value from a list

Replace each occurrence of a marker with the next value from a named list source, in order. This mog fills each <NAME> from a bundled Names list. Use for stitching a column of values back into a template. on_exhausted=leave keeps extra markers if the list runs out.

## Run

```
mog -m fill-from-list <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
Dear <NAME>, welcome.
Dear <NAME>, welcome.
Dear <NAME>, welcome.
```

Output:

```
Dear Ada, welcome.
Dear Bo, welcome.
Dear Cy, welcome.
```

## Pipeline

- `fill_from_list`: Fill each <NAME> from the names list

## Tags

`data` `fill` `codegen` `docs` `redact`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

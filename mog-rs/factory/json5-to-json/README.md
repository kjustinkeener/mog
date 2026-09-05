# JSON5 to strict JSON

Convert JSON5 or JSONC to strict JSON

Clean JSON5 / JSONC into strict JSON in one pass: strip // and /* block */ comments (URL-safe) and remove trailing commas before } or ]. Unquoted keys and single-quoted strings (other JSON5 features) are not converted.

## Run

```
mog -m json5-to-json <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
{
  // the name
  "name": "demo",
  "items": [
    1,
    2,
    3,
  ],
  /* trailing */
  "port": 8080,
}
```

Output:

```
{
  "name": "demo",
  "items": [
    1,
    2,
    3
  ],
  "port": 8080
}
```

## Pipeline

- `run_mog`: Strip comments
- `run_mog`: Remove trailing commas

## Tags

`json` `cleanup` `convert`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

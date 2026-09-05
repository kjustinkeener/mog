# Remove a key from JSON

Delete the value at a dotted path in a JSON doc

Remove the key (or element) at a dotted path in a JSON document, leaving the rest structurally intact. This recipe drops metadata.token, e.g. to strip a secret before sharing. Bounded structural edit, not a full jq.

## Run

```
mog -m remove-json-key <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
{"id": 1, "metadata": {"token": "s3cr3t", "keep": true}}
```

Output:

```
{
  "id": 1,
  "metadata": {
    "keep": true
  }
}
```

## Pipeline

- `json_delete`: Delete metadata.token

## Tags

`json` `strip` `redact`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

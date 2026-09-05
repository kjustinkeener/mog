# Extract JSON from LLM output

Extract JSON from chatty LLM output

Pull the JSON out of a chatty model reply so it can be parsed deterministically: prefers a fenced ```json block, otherwise the first balanced { } / [ ] value, and re-indents it (validating it is real JSON). The agent-native post-processing step for structured model output.

## Run

```
mog -m extract-json-from-llm <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

````
Sure! Here is the record you asked for:

```json
{"name": "Ada Lovelace", "role": "engineer", "active": true}
```

Let me know if you want anything changed.
````

Output:

```
{
  "name": "Ada Lovelace",
  "role": "engineer",
  "active": true
}
```

## Pipeline

- `extract_json`: Extract and pretty-print the JSON

## Tags

`json` `ai` `extract`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

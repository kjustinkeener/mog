# ShareGPT to OpenAI chat format

Convert ShareGPT dataset to OpenAI chat format

Convert a ShareGPT fine-tuning dataset (JSONL, one conversation per line with a conversations array of {from, value}) into OpenAI chat format ({messages: [{role, content}]}). Renames conversations->messages, from->role, value->content, and maps the role values (human->user, gpt->assistant; system is left as-is). JSONL in, JSONL out. The inverse of openai-to-sharegpt.

## Run

```
mog -m sharegpt-to-openai <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
{"conversations": [{"from": "human", "value": "What is 2+2?"}, {"from": "gpt", "value": "4"}]}
{"conversations": [{"from": "system", "value": "Be terse."}, {"from": "human", "value": "Hi"}, {"from": "gpt", "value": "Hello"}]}
```

Output:

```
{"messages":[{"role":"user","content":"What is 2+2?"},{"role":"assistant","content":"4"}]}
{"messages":[{"role":"system","content":"Be terse."},{"role":"user","content":"Hi"},{"role":"assistant","content":"Hello"}]}
```

## Steps

- `json_rename`: conversations -> messages
- `json_rename`: from -> role
- `json_rename`: value -> content
- `replace`: human -> user
- `replace`: gpt -> assistant

## Tags

`ml` `dataset` `fine-tuning` `openai` `convert` `jsonl`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

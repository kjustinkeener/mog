# OpenAI chat format to ShareGPT

Convert OpenAI chat dataset to ShareGPT format

Convert an OpenAI chat fine-tuning dataset (JSONL, one example per line with a messages array of {role, content}) into ShareGPT format ({conversations: [{from, value}]}). Renames messages->conversations, role->from, content->value, and maps role values (user->human, assistant->gpt; system left as-is). JSONL in, JSONL out. Inverse of sharegpt-to-openai.

## Run

```
mog -m openai-to-sharegpt <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
{"messages": [{"role": "user", "content": "What is 2+2?"}, {"role": "assistant", "content": "4"}]}
{"messages": [{"role": "system", "content": "Be terse."}, {"role": "user", "content": "Hi"}, {"role": "assistant", "content": "Hello"}]}
```

Output:

```
{"conversations":[{"from":"human","value":"What is 2+2?"},{"from":"gpt","value":"4"}]}
{"conversations":[{"from":"system","value":"Be terse."},{"from":"human","value":"Hi"},{"from":"gpt","value":"Hello"}]}
```

## Steps

- `json_rename`: messages -> conversations
- `json_rename`: role -> from
- `json_rename`: content -> value
- `replace`: user -> human
- `replace`: assistant -> gpt

## Tags

`ml` `dataset` `openai` `fine-tuning` `convert` `jsonl`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

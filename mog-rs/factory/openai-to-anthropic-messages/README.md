# OpenAI chat messages to Anthropic Messages

Convert OpenAI chat messages to Anthropic Messages API

Convert OpenAI chat records (JSONL, one object per line shaped {messages:[{role, content}]}) into Anthropic Messages API shape ({system?, messages:[{role, content}]}). Extracts a leading system message into Anthropic's top-level system string; records with no leading system message pass through unchanged. Assumes compact JSONL (no spaces after colons), the object is exactly {messages:[...]}, a single system message in first position, and string content. Not handled: multi-part content arrays, tool/function messages, multiple or misplaced system messages, or extra top-level params (model, temperature). Inverse of anthropic-to-openai-messages.

## Run

```
mog -m openai-to-anthropic-messages <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
{"messages":[{"role":"system","content":"You are a terse assistant."},{"role":"user","content":"Hi"},{"role":"assistant","content":"Hello."}]}
{"messages":[{"role":"user","content":"Say \"hi\""},{"role":"assistant","content":"hi"}]}
```

Output:

```
{"system":"You are a terse assistant.","messages":[{"role":"user","content":"Hi"},{"role":"assistant","content":"Hello."}]}
{"messages":[{"role":"user","content":"Say \"hi\""},{"role":"assistant","content":"hi"}]}
```

## Steps

- `replace_regex_multiline`: leading system message -> top-level system string

## Tags

`ml` `dataset` `openai` `anthropic` `chat` `convert` `jsonl`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

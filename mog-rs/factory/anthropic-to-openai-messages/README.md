# Anthropic Messages to OpenAI chat messages

Convert Anthropic Messages to OpenAI chat messages

Convert Anthropic Messages API records (JSONL, one object per line shaped {system?, messages:[{role, content}]}) into OpenAI chat format ({messages:[{role, content}]}). The user/assistant roles and string content are already identical between the two, so the only structural change is hoisting Anthropic's top-level system string into a leading {role:system} message at the front of the messages array. Records without a system field pass through unchanged (already OpenAI-compatible). ASSUMES compact JSONL (no spaces after colons), a record whose object begins with system immediately followed by messages, and string content. NOT handled: multi-part content arrays (content as a list of {type:text,...} blocks), tool_use / tool_result blocks, system supplied as a content-block array, extra top-level params (model, max_tokens), or a system field that is not first. Inverse of openai-to-anthropic-messages.

## Run

```
mog -m anthropic-to-openai-messages <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
{"system":"You are a terse assistant.","messages":[{"role":"user","content":"Hi"},{"role":"assistant","content":"Hello."}]}
{"messages":[{"role":"user","content":"Say \"hi\""},{"role":"assistant","content":"hi"}]}
```

Output:

```
{"messages":[{"role":"system","content":"You are a terse assistant."},{"role":"user","content":"Hi"},{"role":"assistant","content":"Hello."}]}
{"messages":[{"role":"user","content":"Say \"hi\""},{"role":"assistant","content":"hi"}]}
```

## Pipeline

- `replace_regex_multiline`: top-level system string -> leading system message

## Tags

`ml` `dataset` `anthropic` `openai` `chat` `convert` `jsonl`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

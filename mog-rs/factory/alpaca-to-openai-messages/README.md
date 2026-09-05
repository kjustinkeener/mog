# Alpaca to OpenAI chat messages

Convert Alpaca dataset to OpenAI chat messages

Convert an Alpaca instruction dataset (JSONL, one record per line shaped {instruction, input, output}) into OpenAI chat format ({messages:[{role:user, content}, {role:assistant, content}]}). The instruction becomes the user turn and the output the assistant turn; a non-empty input is appended to the user turn after a blank line, and an empty input drops the trailing blank line. Assumes compact JSONL (no spaces after colons) with keys in canonical order instruction, input, output and an explicit input key (empty string when unused). Not handled: records missing the input key, extra fields (system, history), reordered keys, or multi-turn data.

## Run

```
mog -m alpaca-to-openai-messages <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
{"instruction":"Name the capital of France.","input":"","output":"Paris."}
{"instruction":"Translate to French.","input":"Good morning","output":"Bonjour"}
```

Output:

```
{"messages":[{"role":"user","content":"Name the capital of France."},{"role":"assistant","content":"Paris."}]}
{"messages":[{"role":"user","content":"Translate to French.\n\nGood morning"},{"role":"assistant","content":"Bonjour"}]}
```

## Pipeline

- `replace_regex_multiline`: instruction/output only -> user/assistant turns
- `replace_regex_multiline`: merge instruction + input into the user turn

## Tags

`ml` `dataset` `fine-tuning` `openai` `chat` `convert` `jsonl`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

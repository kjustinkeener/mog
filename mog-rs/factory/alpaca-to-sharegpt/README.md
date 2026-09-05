# Alpaca to ShareGPT

Convert Alpaca dataset to ShareGPT format

Convert an Alpaca instruction dataset (JSONL, one record per line shaped {instruction, input, output}) into ShareGPT format ({conversations:[{from:human, value}, {from:gpt, value}]}). The instruction becomes the human turn and the output the gpt turn; a non-empty input is appended to the human turn after a blank line, and an empty input drops the trailing blank line. Assumes compact JSONL (no spaces after colons) with keys in canonical order instruction, input, output and an explicit input key (empty string when unused). Not handled: records missing the input key, extra fields (system, history), reordered keys, or multi-turn data.

## Run

```
mog -m alpaca-to-sharegpt <file>
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
{"conversations":[{"from":"human","value":"Name the capital of France."},{"from":"gpt","value":"Paris."}]}
{"conversations":[{"from":"human","value":"Translate to French.\n\nGood morning"},{"from":"gpt","value":"Bonjour"}]}
```

## Pipeline

- `replace_regex_multiline`: instruction/output only -> human/gpt turns
- `replace_regex_multiline`: merge instruction + input into the human turn

## Tags

`ml` `dataset` `fine-tuning` `convert` `jsonl`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

# Claude transcript to text

Extract readable conversation from a Claude transcript

Strip a Claude Code / AI session transcript (JSONL) down to the readable conversation: keep user and assistant turns, pull the prose (a plain string, or the text blocks of an assistant message), and drop tool calls, tool results, thinking, and system/meta lines. Markdown by default (## role headers); pass -D header= for plain prose. Reads JSON per line; not a general transcript parser.

## Run

```
mog -m claude-transcript-to-text <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
{"type":"custom-title","title":"Session"}
{"type":"user","message":{"role":"user","content":"What is 2+2?"}}
{"type":"assistant","message":{"role":"assistant","content":[{"type":"thinking","thinking":"add them"},{"type":"text","text":"It is 4."}]}}
{"type":"user","message":{"role":"user","content":[{"type":"tool_result","content":"noise"}]}}
{"type":"assistant","message":{"role":"assistant","content":[{"type":"tool_use","name":"calc"}]}}
{"type":"system","subtype":"meta"}
{"type":"user","message":{"role":"user","content":"Thanks!"}}
```

Output:

```
## user

What is 2+2?

## assistant

It is 4.

## user

Thanks!
```

## Pipeline

- `json_filter`: Keep only user and assistant turns (drop system/meta lines).
- `json_filter`: Keep only turns that carry prose (drops tool-only turns and tool results).
- `json_extract`: Emit each turn as text: an optional role header plus the prose.

## Tags

`json` `text` `ai` `extract` `cleanup`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

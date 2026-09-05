# Paste cleanup: Slack / Discord / Teams

Strip chat cruft from pasted Slack/Discord/Teams text

Strip chat-client cruft from a pasted Slack/Discord/Teams conversation, leaving the message text. Removes 'New messages' dividers, 'X replied to a thread' lines, author + timestamp header lines (Name, two or more spaces, then a HH:MM time), standalone timestamp lines, and reaction rows (:emoji: count). Strips a leading [HH:MM]/HH:MM AM timestamp from message lines and inline '(edited)' markers. Chat exports vary: the author-line rule requires two or more spaces before the time to avoid eating real messages, so unusual layouts may leak through or a message ending in a bare time could be dropped.

## Run

```
mog -m paste-cleanup-slack <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
New messages

Alice Smith  10:30 AM
Hey team, the build is green (edited)
:thumbsup: 3
Bob Jones  10:32 AM
Nice work!
Alice Smith replied to a thread: deploy now?
[10:35 AM] shipping it
11:02
Follow-up note
```

Output:

```

Hey team, the build is green
Nice work!
shipping it
Follow-up note
```

## Pipeline

- `remove_lines_matching`: Remove 'New messages' dividers.
- `remove_lines_matching`: Remove 'X replied to a thread' notice lines.
- `remove_lines_matching`: Remove reaction rows: one or more :emoji: each with a count.
- `remove_lines_matching`: Remove 'Name  HH:MM AM' author header lines (two or more spaces before the time).
- `remove_lines_matching`: Remove standalone timestamp lines.
- `replace_regex_multiline`: Strip a leading [HH:MM]/HH:MM AM timestamp from a message line.
- `replace_regex_multiline`: Remove inline '(edited)' markers.

## Tags

`paste` `cleanup` `chat`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

# Mbox to CSV

Index an mbox mailbox as one CSV row per message

Convert an RFC 4155 mbox mailbox into a CSV index of its messages: one row per message with date,from,to,subject,message_id and a header row. Message bodies are dropped, folded headers are unfolded first so long subjects stay intact, header names are matched case-insensitively, and every field is RFC 4180 quoted so commas and quotes inside a subject are safe. A missing header yields an empty cell. RFC 2047 encoded words (=?UTF-8?B?...) are left exactly as they appear; this mog does not decode them. Messages are split on the From_ separator line, so a body line that begins with an unescaped "From " at column 0 would be read as a new message.

## Run

```
mog -m mbox-to-csv <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
From alice@example.com Mon Sep  1 09:14:02 2025
Date: Mon, 1 Sep 2025 09:14:02 +0000
From: Alice Baker <alice@example.com>
To: bob@example.com
Subject: Quarterly planning notes and the follow-up items
 we agreed to review before the next steering meeting
Message-ID: <20250901091402.1001@example.com>
Content-Type: text/plain; charset=us-ascii

Hi Bob,

Here is the excerpt you asked about:

>From the archive we can see the old thread started here.

Thanks,
Alice

```

_(... 21 more line(s))_

Output:

```
date,from,to,subject,message_id
"Mon, 1 Sep 2025 09:14:02 +0000","Alice Baker <alice@example.com>","bob@example.com","Quarterly planning notes and the follow-up items we agreed to review before the next steering meeting","<20250901091402.1001@example.com>"
"Tue, 2 Sep 2025 14:30:11 +0000","Bob Chen <bob@example.com>","alice@example.com, carol@example.com, dave@example.com","Re: budget, staffing, and the ""must have"" list","<20250902143011.1002@example.com>"
"Wed, 3 Sep 2025 07:05:44 +0000","Carol Diaz <carol@example.com>","team@example.com","Server maintenance window",""
```

## Pipeline

- `eol_lf`: Normalize line endings so the header and body rules see plain LF
- `trim_whitespace_right`: Drop trailing whitespace so header values do not carry it into the CSV
- `replace_regex`: Mark each From_ separator line so messages can be told apart
- `replace_regex`: Drop each message body: everything from the first blank line to the next message
- `replace_regex`: Unfold folded headers: a continuation line joins the header above it
- `keep_lines_matching`: Keep only the five headers of interest, plus the message markers
- `replace_regex`: Fold each message's headers onto a single line
- `suffix_lines`: Append empty defaults so a message missing a header still gets an empty cell
- `replace_regex`: Pull the date header into the output row (empty when the message has none)
- `replace_regex`: Pull the from header into the output row (empty when the message has none)
- `replace_regex`: Pull the to header into the output row (empty when the message has none)
- `replace_regex`: Pull the subject header into the output row (empty when the message has none)
- `replace_regex`: Pull the message-id header into the output row (empty when the message has none)
- `replace_regex`: Discard the raw header text now that the five values are extracted
- `replace`: Remove the message marker, leaving just the extracted fields
- `remove_empty_lines`: Drop any blank remnant so no empty row reaches the CSV
- `replace`: Double every embedded quote, as RFC 4180 requires
- `replace`: Turn the field separator into a quoted CSV comma
- `wrap_lines`: Quote the first and last field of every row
- `prepend`: Add the CSV header row

## Tags

`mbox` `email` `csv` `convert` `extract`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

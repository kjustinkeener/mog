# Email headers to CSV

Convert email header blocks to CSV

Turn blank-line-separated blocks of email headers (From:, To:, Subject:, Date:, ...) into a CSV table, one row per message, with a column for each header seen. Splits each line on the first colon. Good for summarizing an mbox-style export or a batch of saved headers.

## Run

```
mog -m email-headers-to-csv <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
From: alice@example.com
To: bob@example.com
Subject: Hello

From: carol@example.com
To: dave@example.com
Subject: Meeting
```

Output:

```
From,To,Subject
alice@example.com,bob@example.com,Hello
carol@example.com,dave@example.com,Meeting
```

## Pipeline

- `records_to_columns`: Pivot each header block into a CSV row

## Tags

`email` `headers` `csv` `convert` `data`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

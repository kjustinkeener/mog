# WebVTT to SRT subtitles

Convert WebVTT captions to SubRip (SRT)

Convert WebVTT (.vtt) captions to SubRip (.srt): drop the WEBVTT header and any leading blank lines, change the timestamp decimal separator from a dot to a comma (00:00:01.000 -> 00:00:01,000), and add the sequential cue-index line that SRT requires. The inverse of srt-to-vtt. Cue settings/positioning and NOTE/STYLE blocks are not handled.

## Run

```
mog -m vtt-to-srt <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
WEBVTT

00:00:01.000 --> 00:00:04.000
Hello world

00:00:05.500 --> 00:00:07.000
Second line
```

Output:

```
1
00:00:01,000 --> 00:00:04,000
Hello world

2
00:00:05,500 --> 00:00:07,000
Second line
```

## Steps

- `remove_lines_matching`: Drop the WEBVTT header line
- `replace_regex`: Drop the leading blank line(s) left by the header
- `replace_regex`: Timestamps: dot decimal -> comma decimal
- `insert_before_matching`: Mark each timing line so it can be numbered
- `stamp_sequence`: Turn the markers into 1, 2, 3, ... cue indices

## Tags

`subtitles` `convert` `interchange`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

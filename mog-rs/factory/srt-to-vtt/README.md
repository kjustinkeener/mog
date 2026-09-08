# SRT subtitles to WebVTT

Convert SubRip (SRT) subtitles to WebVTT

Convert SubRip (.srt) captions to WebVTT (.vtt): add the required WEBVTT header, drop the numeric cue-index lines (SRT has them, VTT does not), and change the timestamp decimal separator from a comma to a dot (00:00:01,000 -> 00:00:01.000). Cue text and blank-line separators are preserved. Mechanical: a caption whose text is a bare number would also be dropped as an index line.

## Run

```
mog -m srt-to-vtt <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
1
00:00:01,000 --> 00:00:04,000
Hello world

2
00:00:05,500 --> 00:00:07,000
Second line
```

Output:

```
WEBVTT

00:00:01.000 --> 00:00:04.000
Hello world

00:00:05.500 --> 00:00:07.000
Second line
```

## Steps

- `remove_lines_matching`: Drop the numeric cue-index lines
- `replace_regex`: Timestamps: comma decimal -> dot decimal
- `prepend`: Add the required WEBVTT header

## Tags

`subtitles` `convert` `interchange`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

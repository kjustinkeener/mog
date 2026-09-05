# Shift subtitle timing

Offset every SRT or WebVTT timestamp by a fixed number of milliseconds

Offset every cue timestamp in a SubRip (.srt) or WebVTT (.vtt) file by a fixed number of milliseconds, so subtitles that run ahead of or behind the audio can be pulled back into sync. Both timestamps of each cue move by the same amount, the interval between cues is preserved, and the file's own decimal separator is kept (a comma for SubRip, a dot for WebVTT); a WebVTT stamp written without an hours part gains one. The offset is the constant offset_ms, negative to pull cues earlier: override it with -D offset_ms=1500 (the demo value is -2500). A cue that would land before zero is pulled to the start of the file rather than written as a negative time, and a cue that would land at or past twenty-four hours is pulled back to the last moment of the day; both cases get a review marker naming what happened. Cue text, cue numbers, WebVTT settings and headers are untouched. Limit: a source timestamp whose hours field is already at or past twenty-four is left alone and marked, because the arithmetic runs over a single day.

## Run

```
mog -m shift-subtitle-timing <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
1
00:00:01,500 --> 00:00:04,000
Hello there, this one starts too early.

2
00:00:59,000 --> 00:01:02,250
This cue crosses the minute mark.

3
00:01:01,000 --> 00:01:03,900
And this one lands a minute earlier.

4
01:00:00,000 --> 01:00:02,000
An hour in, so the hour rolls back too.
```

Output:

```
1
00:00:00,000 --> 00:00:01,500 # WARN(mog): this cue reached back before the start of the file and was pulled to zero
Hello there, this one starts too early.

2
00:00:56,500 --> 00:00:59,750
This cue crosses the minute mark.

3
00:00:58,500 --> 00:01:01,400
And this one lands a minute earlier.

4
00:59:57,500 --> 00:59:59,500
An hour in, so the hour rolls back too.
```

## Pipeline

- `flag_matching`: A cue already past the day boundary cannot be shifted
- `replace_regex`: WebVTT start stamp without hours gains an hours part
- `replace_regex`: WebVTT end stamp without hours gains an hours part
- `replace_regex`: Split each cue's two stamps onto working lines, keeping the separator and any settings
- `iso_to_epoch`: Seconds since the start of the day
- `replace_regex`: Fold the milliseconds back onto the seconds
- `arithmetic`: Add the offset to every cue time
- `replace_regex`: A time before zero is pulled to the start of the file
- `replace_regex`: Widen a one-digit result
- `replace_regex`: Widen a two-digit result
- `replace_regex`: Widen a three-digit result
- `replace_regex`: Peel the milliseconds off again
- `epoch_to_iso`: Seconds back to a clock time
- `replace`: Drop the carrier date
- `replace_regex`: A time past the day boundary is pulled back to the last moment of the day
- `replace_regex`: Put each cue's two stamps back on one line
- `flag_matching`: Mark a cue that was pulled to the start of the file
- `flag_matching`: Mark a cue that was pulled back to the end of the day
- `replace_regex`: Remove the working markers

## Tags

`subtitles` `srt` `webvtt` `time` `sync`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

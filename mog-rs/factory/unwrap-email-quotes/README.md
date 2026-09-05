# Unwrap hard-wrapped text, quote-aware

Join wrapped lines back to one line per paragraph, keeping > prefixes

Join each hard-wrapped paragraph back into one long line, the inverse of a fixed-width reflow. With prefix_aware on, a leading quote or comment marker (> , #, //, --) is detected on the paragraph and its wrapped lines are joined under a single prefix, so a '> ' quoted block collapses to one '> ' line instead of concatenating the markers. Blank lines between paragraphs are preserved, so paragraph boundaries survive. Change separator on the step if lines should join with something other than a single space.

## Run

```
mog -m unwrap-email-quotes <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
> This is a fairly long quoted line from the
> original message that was hard-wrapped and
> should collapse back to a single quoted line.

Thanks for the note. Here is my reply, which was
also wrapped across a few lines and should join
back into one paragraph.
```

Output:

```
> This is a fairly long quoted line from the original message that was hard-wrapped and should collapse back to a single quoted line.

Thanks for the note. Here is my reply, which was also wrapped across a few lines and should join back into one paragraph.
```

## Pipeline

- `unwrap_text`: Join each paragraph's wrapped lines under its quote/comment prefix

## Tags

`whitespace` `unwrap` `reflow` `email` `markdown` `single-action`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

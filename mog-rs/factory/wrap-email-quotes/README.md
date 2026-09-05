# Hard-wrap text to a width, quote-aware

Reflow prose to a fixed width, keeping > quote prefixes

Reflow prose to a fixed column width (like fmt or a mail composer's format=flowed), keeping paragraphs separate. With prefix_aware on, a leading quote or comment marker (> , #, //, --) is detected on each paragraph and carried onto every wrapped line, so a long email reply quoted with '> ' re-wraps as '> ' lines instead of one run-on. Blank lines between paragraphs are preserved verbatim. Set width on the step (default 72 here); long unbreakable words are left intact unless break_long_words is enabled.

## Run

```
mog -m wrap-email-quotes <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
> This is a fairly long quoted line from the original message that should be reflowed.
> A short quoted follow-up that joins onto it.

Thanks for the note. Here is my reply, which is also long enough that it needs wrapping across a few lines at the chosen width.
```

Output:

```
> This is a fairly long quoted line from the
> original message that should be reflowed. A
> short quoted follow-up that joins onto it.

Thanks for the note. Here is my reply, which is
also long enough that it needs wrapping across a
few lines at the chosen width.
```

## Pipeline

- `wrap_text`: Wrap each paragraph to the width, carrying any quote/comment prefix

## Tags

`whitespace` `wrap` `reflow` `email` `markdown` `single-action`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

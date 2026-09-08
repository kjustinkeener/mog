# AI Glyph Cleanup

Strip machine-text glyphs down to plain ASCII

Strip the Unicode glyphs that mark machine-generated or word-processor text and leave plain ASCII: every dash variant (as dash-cleanup), curly quotes, ellipsis, non-breaking and zero-width characters (as paste-cleanup), narrow and ideographic spaces, word joiners, direction marks, line and paragraph separators, decorative bullet glyphs at the start of a line, arrows, math and fraction symbols, trademark marks, ligatures, and decorative status emoji. Use it before committing or publishing text that came from a chat model, a PDF, or a word processor.

## Run

```
mog -m ai-glyph-cleanup <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
✅ Done – the “smart” bits…
• first item
• a → b, x × 2, ½ of it
café ― 3–4 ≤ 5 ™
```

Output:

```
Done - the "smart" bits...
- first item
- a -> b, x x 2, 1/2 of it
café - 3-4 <= 5 (TM)
```

## Steps

- `replace_regex`: En dash inside a word or number is a range: make it a tight hyphen
- `replace_map`: Fold the remaining dash variants onto forms emdash-cleanup handles
- `replace_regex`: Line-opening dash becomes a plain hyphen bullet
- `replace_regex`: Trailing dash at end of line is dropped with its leading space
- `replace_regex`: Mid-line dash becomes a spaced hyphen
- `replace_map`: Map smart typography to ASCII equivalents in a single pass
- `replace_map`: Exotic spaces to a normal space; invisible formatting characters removed
- `replace_regex`: Decorative bullet glyphs at the start of a line become a plain hyphen bullet
- `replace_map`: Arrows to ASCII
- `replace_map`: Math, fraction, and symbol glyphs to ASCII
- `replace_regex`: Decorative status emoji and the space after them are removed

## Tags

`text` `cleanup` `unicode` `prose` `ascii`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

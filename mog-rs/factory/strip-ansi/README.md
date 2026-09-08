# Strip ANSI

Remove ANSI/VT escape and color codes from terminal output

Remove ANSI/VT escape sequences (SGR color codes, cursor moves, OSC titles) from captured terminal or log output. grep only matches lines; this rewrites them.

## Run

```
mog -m strip-ansi <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
plain, no codes
[31merror:[0m disk full
[1;38;5;208mwarn[0m 256-color
[38;2;0;128;255mtruecolor[0m rgb
title:]0;window title after OSC
array[0] = x[1] literal brackets stay
[2K[1Gcursor stuff gone
```

Output:

```
plain, no codes
error: disk full
warn 256-color
truecolor rgb
title: after OSC
array[0] = x[1] literal brackets stay
cursor stuff gone
```

## Steps

- `replace_regex`: Strip CSI sequences (SGR colors, cursor moves, clears)
- `replace_regex`: Strip OSC sequences (window/title), terminated by BEL or ST

## Tags

`terminal` `color` `log` `strip`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

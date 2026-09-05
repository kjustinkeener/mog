# No Dash Guard

Fail the run if any Unicode dash is present

Fail the run when any Unicode dash (em dash, horizontal bar, en dash, two-em or three-em dash, figure dash, non-breaking hyphen, or minus sign) is present, and pass the text through unchanged otherwise. Nothing is rewritten: this is the gate, so pair it with dash-cleanup, which does the rewrite. Exits nonzero so a pre-commit hook or CI step can block the content.

## Run

```
mog -m no-dash-guard <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
plain ascii - all good
nothing exotic here
```

Output:

```
plain ascii - all good
nothing exotic here
```

## Pipeline

- `assert`: No Unicode dash may survive

## Tags

`text` `review` `unicode` `guard` `audit`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

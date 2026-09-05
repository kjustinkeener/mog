# ROT13

Apply the ROT13 letter cipher (self-inverse)

Apply the classic ROT13 letter substitution to ASCII letters. Self-inverse: run it again to decode. Digits, punctuation, whitespace, and non-ASCII bytes pass through untouched.

## Run

```
mog -m rot13 <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
Hello, World! 123
The quick brown fox.
```

Output:

```
Uryyb, Jbeyq! 123
Gur dhvpx oebja sbk.
```

## Pipeline

- `rot13`

## Tags

`encode` `redact` `single-action`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

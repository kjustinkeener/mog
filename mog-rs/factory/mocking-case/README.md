# Mocking (random) case

Randomly upper/lower each letter (seeded, reproducible)

Randomly upper- or lower-case each letter. A fixed seed makes the output reproducible, so the golden is stable. Mostly a novelty (the 'mocking SpongeBob' effect); the seed keeps it deterministic.

## Run

```
mog -m mocking-case <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
this is a normal sentence
```

Output:

```
ThIs iS a nOrMAl seNtEnCE
```

## Steps

- `random_case`: Randomly recase each letter

## Tags

`case` `text` `random` `ml` `single-action`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

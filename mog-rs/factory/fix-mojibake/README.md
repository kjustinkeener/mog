# Repair mojibake (double-encoded UTF-8)

Repair mojibake from double-encoded UTF-8

Fix mojibake: text where UTF-8 was misread as Latin-1 and re-encoded, so 'e-acute' shows up as 'A-tilde'-style garbage. Restores the intended characters using a tested mapping of the common sequences. Best on text that is uniformly double-encoded.

## Run

```
mog -m fix-mojibake <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
CafÃ© and rÃ©sumÃ© with a naÃ¯ve soupÃ§on.
```

Output:

```
Café and résumé with a naïve soupçon.
```

## Pipeline

- `fix_mojibake`: Repair double-encoded UTF-8 sequences

## Tags

`text` `unicode` `cleanup` `single-action`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

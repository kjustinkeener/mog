# CSV to TSV

Convert CSV to tab-separated values

Convert comma-separated values to tab-separated values, quote-aware: a field that contained the comma delimiter is unwrapped (tabs rarely need quoting), so "a,b" becomes a single a,b cell joined by tabs. The inverse of tsv-to-csv. Single-line fields only (an embedded newline inside a quoted field is not supported).

## Run

```
mog -m csv-to-tsv <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
name,city,note
Ada,"London, UK",first
Bob,Paris,second
```

Output:

```
name	city	note
Ada	London, UK	first
Bob	Paris	second
```

## Steps

- `change_delimiter`: Re-delimit each line from comma to tab

## Tags

`csv` `tsv` `convert` `data` `fragment`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

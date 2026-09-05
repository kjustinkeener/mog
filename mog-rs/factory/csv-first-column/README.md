# Keep the first CSV column

Keep only the first CSV column

Select just the first column of a CSV, dropping the rest (quote-aware). Handy for pulling a single key/id column out of a table. Change the fields option on the step to keep or reorder other columns (e.g. "1,3").

## Run

```
mog -m csv-first-column <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
id,name,role
1,Ada,Lead
2,Bob,Eng
```

Output:

```
id
1
2
```

## Pipeline

- `cut_fields`: Keep column 1

## Tags

`csv` `column` `extract` `query` `data`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

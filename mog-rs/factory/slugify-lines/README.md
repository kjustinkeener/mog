# Slugify each line

Turn each line into a URL-friendly slug

Turn each line (a title or heading) into a URL/filename-friendly slug: lowercase, spaces and punctuation collapsed to single hyphens, leading/trailing hyphens trimmed. Handy for generating anchors, slugs, or file names from a list of titles.

## Run

```
mog -m slugify-lines <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
My Great Post!
Another Title (2024)
Hello, World
```

Output:

```
my-great-post
another-title-2024
hello-world
```

## Pipeline

- `slugify`: Slugify each line

## Tags

`case` `url` `identifier` `text` `convert`

---
<!-- Generated from the recipe by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

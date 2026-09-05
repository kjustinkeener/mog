# Wrap raw SQL as a dbt model

Wrap raw SQL as a dbt model

Scaffold a dbt model from a raw SELECT: prepend a {{ config(materialized='view') }} block, and flag each hardcoded schema.table reference in a FROM/JOIN so you can replace it with ref() or source(). It marks the references rather than rewriting them, since it cannot know which are models vs sources.

## Run

```
mog -m dbt-model-from-raw-sql <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
SELECT
  u.id,
  u.name,
  o.total
FROM analytics.users u
JOIN analytics.orders o ON o.user_id = u.id
WHERE u.active = true
```

Output:

```
{{ config(materialized='view') }}

SELECT
  u.id,
  u.name,
  o.total
FROM analytics.users u -- FIXME(mog): replace with ref() or source()
JOIN analytics.orders o ON o.user_id = u.id -- FIXME(mog): replace with ref() or source()
WHERE u.active = true
```

## Pipeline

- `prepend`: Add the dbt config block
- `flag_matching`: Flag hardcoded schema.table refs for ref()/source()

## Tags

`sql` `ml` `analytics` `codemod`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

{{ config(materialized='view') }}

SELECT
  u.id,
  u.name,
  o.total
FROM analytics.users u -- FIXME(mog): replace with ref() or source()
JOIN analytics.orders o ON o.user_id = u.id -- FIXME(mog): replace with ref() or source()
WHERE u.active = true

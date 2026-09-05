SELECT
  u.id,
  u.name,
  o.total
FROM analytics.users u
JOIN analytics.orders o ON o.user_id = u.id
WHERE u.active = true

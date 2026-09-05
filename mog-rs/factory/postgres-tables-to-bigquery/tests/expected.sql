CREATE TABLE `users` (
  id INT64,
  name STRING,
  active BOOL,
  data JSON,
  created TIMESTAMP,
  local_ts DATETIME
);
SELECT CAST(id AS STRING) FROM `users`;

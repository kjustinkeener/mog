CREATE TABLE `users` (
  id INT64,
  name STRING,
  score FLOAT64,
  active BOOL,
  created TIMESTAMP,
  local_ts DATETIME,
  raw BYTES
);
SELECT CAST(id AS STRING) FROM `users`;

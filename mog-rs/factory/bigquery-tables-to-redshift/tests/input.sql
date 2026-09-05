CREATE TABLE `events` (
  id INT64,
  name STRING,
  amount NUMERIC,
  created TIMESTAMP,
  local_ts DATETIME,
  payload JSON
);

CREATE TABLE `users` (
  id INT64,
  name STRING,
  score FLOAT64,
  active BOOL,
  data BYTES
);
SELECT SAFE_CAST(score AS INT64) FROM `users`;

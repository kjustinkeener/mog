CREATE TABLE `users` (
  id BIGINT,
  name TEXT,
  score DOUBLE,
  active TINYINT(1),
  data BLOB
);
SELECT CAST(score AS BIGINT) FROM `users`;

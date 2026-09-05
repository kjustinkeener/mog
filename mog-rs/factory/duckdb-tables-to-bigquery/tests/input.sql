CREATE TABLE "users" (
  id BIGINT,
  name VARCHAR,
  score DOUBLE,
  active BOOLEAN,
  created TIMESTAMPTZ,
  local_ts TIMESTAMP,
  raw BLOB
);
SELECT id::VARCHAR FROM "users";

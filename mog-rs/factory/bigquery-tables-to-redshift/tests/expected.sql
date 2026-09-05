CREATE TABLE "events" (
  id BIGINT,
  name VARCHAR,
  amount DECIMAL,
  created TIMESTAMPTZ,
  local_ts TIMESTAMP,
  payload SUPER
);

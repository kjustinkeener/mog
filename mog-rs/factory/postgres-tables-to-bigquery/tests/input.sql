CREATE TABLE "users" (
  id SERIAL,
  name TEXT,
  active BOOLEAN,
  data JSONB,
  created TIMESTAMPTZ,
  local_ts TIMESTAMP
);
SELECT id::TEXT FROM "users";

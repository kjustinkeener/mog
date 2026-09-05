CREATE TABLE events(
  id BIGINT,
  name VARCHAR,
  payload JSON,
  amount DECIMAL(18,2),
  created_at TIMESTAMP,
  updated_at TIMESTAMPTZ,
  tags VARCHAR[]
);

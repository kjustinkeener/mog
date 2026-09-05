CREATE TABLE customers (
  id BIGINT,
  name VARCHAR,
  balance DECIMAL(18,2),
  attrs JSON,
  created_at TIMESTAMP,
  updated_at TIMESTAMP WITH TIME ZONE
);

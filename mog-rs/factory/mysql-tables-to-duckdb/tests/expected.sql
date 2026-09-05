CREATE TABLE products (
-- TODO(mog): MySQL dropped; create a DuckDB SEQUENCE and DEFAULT nextval(...) if needed
  id bigint NOT NULL,
  sku varchar(40) NOT NULL,
  name varchar(200),
  description text,
  price decimal(18,2) DEFAULT '0.00',
  in_stock BOOLEAN DEFAULT '1',
  attrs JSON,
  status VARCHAR DEFAULT 'active', -- TODO(mog): MySQL ENUM mapped to VARCHAR; the allowed-value constraint was lost
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMPTZ NULL DEFAULT CURRENT_TIMESTAMP, -- TODO(mog): MySQL ON UPDATE auto-refresh dropped; no portable equivalent (use a trigger/stream/task)
  PRIMARY KEY (id)
)

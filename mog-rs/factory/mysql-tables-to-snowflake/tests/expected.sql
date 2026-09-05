CREATE TABLE products (
  id bigint NOT NULL AUTOINCREMENT,
  sku varchar(40) NOT NULL,
  name varchar(200),
  description VARCHAR,
  price decimal(18,2) DEFAULT '0.00',
  in_stock BOOLEAN DEFAULT '1',
  attrs VARIANT,
  status VARCHAR DEFAULT 'active', -- TODO(mog): MySQL ENUM converted to VARCHAR; the allowed-value constraint was lost (add a CHECK if needed)
  created_at TIMESTAMP_NTZ DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP_LTZ NULL DEFAULT CURRENT_TIMESTAMP, -- TODO(mog): MySQL ON UPDATE auto-refresh dropped; no portable equivalent (use a trigger/stream/task)
  PRIMARY KEY (id)
)

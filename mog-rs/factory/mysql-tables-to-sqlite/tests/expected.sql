CREATE TABLE products (
-- TODO(mog): MySQL dropped; SQLite needs INTEGER PRIMARY KEY AUTOINCREMENT
  id bigint NOT NULL,
  sku varchar(40) NOT NULL,
  name varchar(200),
  description text,
  price decimal(18,2) DEFAULT '0.00',
  in_stock INTEGER DEFAULT '1',
  attrs TEXT,
  status TEXT DEFAULT 'active',
  created_at datetime DEFAULT CURRENT_TIMESTAMP,
  updated_at timestamp NULL DEFAULT CURRENT_TIMESTAMP, -- TODO(mog): MySQL ON UPDATE auto-refresh dropped; no portable equivalent (use a trigger/stream/task)
  PRIMARY KEY (id)
)

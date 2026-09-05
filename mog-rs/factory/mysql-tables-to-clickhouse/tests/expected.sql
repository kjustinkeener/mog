CREATE TABLE my2ch_orders (
  id Int64, -- TODO(mog): MySQL AUTO_INCREMENT has no ClickHouse equivalent (use a generated UUID, a sequence table, or an application-supplied id)
  order_no String,
  customer_name Nullable(String),
  notes Nullable(String),
  amount Decimal(18,2) DEFAULT '0.00',
  is_paid UInt8 DEFAULT '0',
  priority Nullable(Int8) DEFAULT '5',
  quantity Int32 DEFAULT '1',
  discount_pct Nullable(Float64),
  status String DEFAULT 'pending', -- TODO(mog): MySQL ENUM mapped to String; the allowed-value constraint was lost (use a ClickHouse Enum8/Enum16 or a CHECK)
  order_date Nullable(Date),
  created_at DateTime64(3) DEFAULT CURRENT_TIMESTAMP,
  updated_at Nullable(DateTime64(3)) DEFAULT CURRENT_TIMESTAMP -- TODO(mog): MySQL ON UPDATE auto-refresh dropped; no portable equivalent (use a trigger/stream/task)
  -- TODO(mog): MySQL UNIQUE KEY uq_order_no (order_no) dropped; ClickHouse does not enforce uniqueness (dedupe with ReplacingMergeTree or enforce upstream)
  -- TODO(mog): MySQL secondary index idx_customer (customer_name) dropped; ClickHouse uses the ORDER BY key and data-skipping indexes instead
) ENGINE = MergeTree
ORDER BY (id)

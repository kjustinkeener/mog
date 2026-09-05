CREATE TABLE customers (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  balance REAL,
  data BLOB,
  is_active INTEGER,
  created_at TEXT
);

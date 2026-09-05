SELECT o.id AS order_id, c.name AS customer, o.total, 'paid' AS status FROM orders o JOIN customers c ON o.customer_id = c.id

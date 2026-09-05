-- Database: lab

CREATE VIEW v_viewdemo
AS
SELECT
    order_id,
    customer_name,
    COALESCE(region, 'UNKNOWN') AS region_clean,
    quantity * unit_price AS gross_amount,
    quantity * unit_price * (1 - COALESCE(discount, 0)) AS net_amount,
    LENGTH(customer_name) AS name_len,
    now() AS snapshot_at
FROM viewdemo_orders
WHERE status_code = 1 AND quantity > 0
ORDER BY order_date DESC
LIMIT 100;


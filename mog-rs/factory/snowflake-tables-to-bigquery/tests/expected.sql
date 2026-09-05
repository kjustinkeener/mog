CREATE OR REPLACE TABLE `analytics`.`orders` (
  `order_id` NUMERIC(38,0),
  `customer_id` NUMERIC,
  `amount` NUMERIC(10,2),
  `created_at` DATETIME,
  `updated_at` TIMESTAMP,
  `notes` STRING(500),
  `payload` JSON,
  `is_active` BOOL
);

SELECT
  `order_id`,
  IFNULL(`amount`, 0) AS amount,
  IF(`is_active`, 1, 0) AS active_flag,
  CAST(`created_at` AS DATE) AS created_date
FROM `analytics`.`orders`;

CREATE TABLE `my2ch_orders` (
  `id` bigint NOT NULL AUTO_INCREMENT,
  `order_no` varchar(40) NOT NULL,
  `customer_name` varchar(200) DEFAULT NULL,
  `notes` text,
  `amount` decimal(18,2) NOT NULL DEFAULT '0.00',
  `is_paid` tinyint(1) NOT NULL DEFAULT '0',
  `priority` tinyint DEFAULT '5',
  `quantity` int NOT NULL DEFAULT '1',
  `discount_pct` double DEFAULT NULL,
  `status` enum('pending','shipped','cancelled') NOT NULL DEFAULT 'pending',
  `order_date` date DEFAULT NULL,
  `created_at` datetime NOT NULL DEFAULT CURRENT_TIMESTAMP,
  `updated_at` timestamp NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`),
  UNIQUE KEY `uq_order_no` (`order_no`),
  KEY `idx_customer` (`customer_name`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci

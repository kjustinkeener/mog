CREATE TABLE `products` (
  `id` bigint NOT NULL AUTO_INCREMENT,
  `sku` varchar(40) NOT NULL,
  `name` varchar(200) DEFAULT NULL,
  `description` text,
  `price` decimal(18,2) DEFAULT '0.00',
  `in_stock` tinyint(1) DEFAULT '1',
  `attrs` json DEFAULT NULL,
  `status` enum('active','discontinued') DEFAULT 'active',
  `created_at` datetime DEFAULT CURRENT_TIMESTAMP,
  `updated_at` timestamp NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_0900_ai_ci

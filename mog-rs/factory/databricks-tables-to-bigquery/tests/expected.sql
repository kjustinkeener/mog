CREATE TABLE `main`.`events` (
  `event_id` INT64,
  `count` INT64,
  `code` INT64,
  `flag` INT64,
  `amount` NUMERIC(10,2),
  `ratio` FLOAT64,
  `name` STRING,
  `tags` JSON,
  `attrs` JSON,
  `raw` BYTES,
  `is_active` BOOL,
  `created_at` TIMESTAMP
);

SELECT
  `event_id`,
  IFNULL(`name`, 'x') AS name
FROM `main`.`events`;

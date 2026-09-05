CREATE TABLE `main`.`events` (
  `event_id` LONG,
  `count` INT,
  `code` SHORT,
  `flag` BYTE,
  `amount` DECIMAL(10,2),
  `ratio` DOUBLE,
  `name` STRING,
  `tags` ARRAY<STRING>,
  `attrs` MAP<STRING, STRING>,
  `raw` BINARY,
  `is_active` BOOLEAN,
  `created_at` TIMESTAMP
)
USING DELTA;

SELECT
  `event_id`,
  NVL(`name`, 'x') AS name
FROM `main`.`events`;

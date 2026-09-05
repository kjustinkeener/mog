-- BigQuery DDL export
-- Source: SELECT ddl FROM `mog-demo.analytics.INFORMATION_SCHEMA.TABLES`

CREATE SCHEMA IF NOT EXISTS `mog-demo.analytics`;

DROP TABLE IF EXISTS `mog-demo.analytics.bqgo_scratch`;

-- Table: bqgo_actor

CREATE OR REPLACE TABLE `mog-demo.analytics.bqgo_actor`
(
    `actor_id` INT64 NOT NULL,
    `first_name` STRING NOT NULL,
    `last_name` STRING NOT NULL,
    `last_update` TIMESTAMP NOT NULL
);

-- Table: bqgo_staff

CREATE OR REPLACE TABLE `mog-demo.analytics.bqgo_staff`
(
    `staff_id` INT64 NOT NULL,
    `first_name` STRING NOT NULL,
    `last_name` STRING NOT NULL,
    `address_id` INT64 NOT NULL,
    `email` STRING OPTIONS(description="Contact email"),
    `store_id` INT64 NOT NULL,
    `active` BOOL DEFAULT true NOT NULL,
    `username` STRING NOT NULL,
    `password` STRING,
    `last_update` TIMESTAMP NOT NULL,
    `picture` BYTES
)
PARTITION BY DATE(last_update)
CLUSTER BY store_id
OPTIONS(
  description="Store staff",
  labels=[("env", "prod")]
);

-- Table: bqgo_widget

CREATE OR REPLACE TABLE `mog-demo.analytics.bqgo_widget`
(
    `widget_id` INT64 NOT NULL,
    `small_count` INT64 NOT NULL,
    `big_count` INT64,
    `unit_price` NUMERIC(10, 2) NOT NULL,
    `huge_total` BIGNUMERIC(38, 9),
    `ratio` FLOAT64,
    `sku` STRING(45) NOT NULL,
    `grade` STRING(1),
    `description` STRING,
    `is_active` BOOL NOT NULL,
    `created_on` DATE NOT NULL,
    `open_time` TIME,
    `made_at` DATETIME NOT NULL,
    `updated_at` TIMESTAMP DEFAULT CURRENT_TIMESTAMP() NOT NULL,
    `thumbnail` BYTES,
    `payload` JSON,
    `shipping` STRUCT<street STRING, city STRING, zip INT64>,
    `tags` ARRAY<STRING>,
    `scores` ARRAY<FLOAT64>
);

-- View: bqgo_active_staff

CREATE OR REPLACE VIEW `mog-demo.analytics.bqgo_active_staff` AS
SELECT staff_id, username
FROM `mog-demo.analytics.bqgo_staff`
WHERE active;

ALTER TABLE `mog-demo.analytics.bqgo_actor` SET OPTIONS (description="Actors");

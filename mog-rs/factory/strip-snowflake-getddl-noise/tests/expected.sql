create or replace TABLE CUSTOMERS (
	ID NUMBER(38,0) NOT NULL autoincrement start 1 increment 1,
	NAME VARCHAR(200),
	EMAIL VARCHAR(255),
	BALANCE NUMBER(18,2),
	ATTRS VARIANT,
	CREATED_AT TIMESTAMP_NTZ(9),
	UPDATED_AT TIMESTAMP_TZ(9),
	REGION VARCHAR(50),
	constraint PK_CUSTOMERS primary key (ID)
)
-- TODO(mog): dropped Snowflake CLUSTER BY (REGION) (set target clustering/partitioning manually)

CREATE TABLE ORA2SF_ORDERS 
   (	ORDER_ID NUMBER(38,0) IDENTITY START 1000 INCREMENT 1 NOT NULL, 
	CUSTOMER_ID NUMBER(38,0) NOT NULL, 
	ORDER_NO VARCHAR(30) NOT NULL, 
	STATUS VARCHAR(20) DEFAULT 'PENDING' NOT NULL, 
	IS_ACTIVE NUMBER(1,0) DEFAULT 1 NOT NULL, 
	QUANTITY NUMBER(10,0) DEFAULT 0, 
	UNIT_PRICE NUMBER(12,2) NOT NULL, 
	DISCOUNT_PCT NUMBER(5,4), 
	TOTAL_AMOUNT NUMBER(14,2) AS (QUANTITY*UNIT_PRICE) ,  -- TODO(mog): oracle virtual column rewritten as a snowflake computed column; verify the expression is deterministic and supported
	DESCRIPTION VARCHAR,  -- TODO(mog): oracle lob downcast to a bounded snowflake type; large-object semantics differ
	NOTES VARCHAR(4000), 
	PAYLOAD BINARY,  -- TODO(mog): oracle lob downcast to a bounded snowflake type; large-object semantics differ
	CHECKSUM BINARY, 
	CREATED_AT TIMESTAMP_NTZ DEFAULT CURRENT_TIMESTAMP() NOT NULL,  -- TODO(mog): oracle date+time column mapped to a zoneless snowflake timestamp; confirm timezone handling
	UPDATED_AT TIMESTAMP_NTZ(6), 
	SHIP_TS TIMESTAMP_TZ, 
	LOCAL_TS TIMESTAMP_LTZ, 
	 CONSTRAINT CK_ORA2SF_QTY CHECK (quantity >= 0),  -- TODO(mog): snowflake parses but does not enforce check constraints; enforce this invariant in etl
	 CONSTRAINT PK_ORA2SF_ORDERS PRIMARY KEY (ORDER_ID), 
	 CONSTRAINT UQ_ORA2SF_ORDER_NO UNIQUE (ORDER_NO)
  ) 
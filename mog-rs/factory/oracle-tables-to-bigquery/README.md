# Oracle (SQL Developer GET_DDL) to BigQuery table DDL

Convert Oracle table DDL to BigQuery

Convert an Oracle DBMS_METADATA.GET_DDL table export to runnable BigQuery standard-SQL DDL. Strips the GET_DDL scaffolding (schema qualifier, double-quote quoting, ENABLE, USING INDEX, storage tail) and maps types: NUMBER(p,0)->INT64, NUMBER(p,s)->NUMERIC(p,s), bare NUMBER->NUMERIC, VARCHAR2/NVARCHAR2/CHAR/NCHAR/CLOB/NCLOB/LONG->STRING, BINARY_FLOAT/BINARY_DOUBLE/FLOAT->FLOAT64, RAW/LONG RAW/BLOB->BYTES, ROWID/UROWID->STRING, TIMESTAMP WITH [LOCAL] TIME ZONE->TIMESTAMP, bare/precision TIMESTAMP->DATETIME, DATE->DATETIME (Oracle DATE carries a time-of-day). Appends NOT ENFORCED to the inline PRIMARY KEY and switches CREATE TABLE to CREATE OR REPLACE TABLE. Converts table DDL only, not sequences/triggers/PL-SQL; assumes the standard GET_DDL layout.

## Run

```
mog -m oracle-tables-to-bigquery <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```

  CREATE TABLE "APP"."CUSTOMERS"
   (	"ID" NUMBER(38,0) NOT NULL ENABLE,
	"NAME" VARCHAR2(200),
	"CODE" NVARCHAR2(50),
	"GRADE" CHAR(1),
	"DESCRIPTION" CLOB,
	"BALANCE" NUMBER(18,2),
	"RATE" BINARY_DOUBLE,
	"IS_ACTIVE" NUMBER(1,0),
	"PAYLOAD" BLOB,
	"CHECKSUM" RAW(16),
	"CREATED_AT" DATE,
	"UPDATED_AT" TIMESTAMP (6),
	"EVENT_AT" TIMESTAMP (6) WITH TIME ZONE,
	 CONSTRAINT "PK_CUSTOMERS" PRIMARY KEY ("ID")
  USING INDEX  ENABLE
   )
```

Output:

```
CREATE OR REPLACE TABLE CUSTOMERS
   (	ID INT64 NOT NULL,
	NAME STRING(200),
	CODE STRING(50),
	GRADE STRING(1),
	DESCRIPTION STRING,
	BALANCE NUMERIC(18,2),
	RATE FLOAT64,
	IS_ACTIVE INT64,
	PAYLOAD BYTES,
	CHECKSUM BYTES(16),
	CREATED_AT DATETIME,
	UPDATED_AT DATETIME,
	EVENT_AT TIMESTAMP,
	 CONSTRAINT PK_CUSTOMERS PRIMARY KEY (ID) NOT ENFORCED
  )
```

## Pipeline

- `run_mog`
- `replace_regex`: NUMBER(p,0) integer -> INT64 (BigQuery's 64-bit integer; covers NUMBER(1,0) flags too).
- `replace_regex`: NUMBER(p,s) with a scale -> NUMERIC(p,s).
- `replace_regex`: bare NUMBER -> NUMERIC.
- `replace_regex`: VARCHAR2(n)/NVARCHAR2(n) -> STRING(n) (BigQuery STRING is Unicode; the length is valid parameterized-STRING syntax).
- `replace_regex`: CLOB/NCLOB/LONG (character LOB) -> STRING.
- `replace_regex`: CHAR(n)/NCHAR(n) -> STRING(n) (run after the VARCHAR2 rules so it does not bite them).
- `replace_regex`: LONG RAW / RAW(n) / BLOB (binary) -> BYTES.
- `replace_regex`: BINARY_FLOAT / BINARY_DOUBLE / FLOAT -> FLOAT64.
- `replace_regex`: ROWID / UROWID -> STRING (no BigQuery physical-address type).
- `replace_regex`: TIMESTAMP(n)? WITH [LOCAL] TIME ZONE (an instant) -> a sentinel, so the bare-TIMESTAMP rule below does not turn it into DATETIME. Resolved to BigQuery TIMESTAMP at the end.
- `replace_regex`: Remaining TIMESTAMP(n) / bare TIMESTAMP (wall-clock) -> DATETIME (BigQuery DATETIME has no precision argument).
- `replace_regex`: Oracle DATE carries a time-of-day -> DATETIME (not DATE).
- `replace`: Resolve the with-time-zone sentinel to BigQuery TIMESTAMP (an absolute instant).
- `replace_regex`: BigQuery accepts only PRIMARY KEY ... NOT ENFORCED: append NOT ENFORCED to the inline key.
- `replace`: BigQuery redeploy idiom: CREATE TABLE -> CREATE OR REPLACE TABLE (idempotent).

## Tags

`sql` `oracle` `bigquery` `warehouse` `convert` `migration`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

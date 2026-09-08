# Oracle GET_DDL to Snowflake

Convert an Oracle DBMS_METADATA.GET_DDL export to Snowflake DDL

Convert an Oracle DBMS_METADATA.GET_DDL table export (storage suppressed) to Snowflake DDL. Strips the GET_DDL boilerplate, rewrites an Oracle GENERATED ALWAYS AS IDENTITY column into Snowflake IDENTITY START n INCREMENT n, converts a virtual column (GENERATED ALWAYS AS (expr) VIRTUAL) into a Snowflake computed column (type AS (expr)), maps SYSDATE/SYSTIMESTAMP to CURRENT_TIMESTAMP(), and maps types: VARCHAR2(n)->VARCHAR(n), CLOB->VARCHAR, RAW(n)->BINARY, BLOB->BINARY, TIMESTAMP(n) WITH LOCAL TIME ZONE->TIMESTAMP_LTZ, TIMESTAMP(n) WITH TIME ZONE->TIMESTAMP_TZ, TIMESTAMP(n)->TIMESTAMP_NTZ(n), DATE->TIMESTAMP_NTZ (Oracle DATE carries a time component). NUMBER(p,s) kept as-is; PRIMARY KEY/UNIQUE/CHECK/DEFAULT/NOT NULL preserved. Non-portable or semantically-shifted features (zoneless DATE, LOB downcasts, virtual column, parsed-but-unenforced CHECK) are annotated with -- TODO(mog) lines. Execution-validated against a real Snowflake engine (fakesnow).

## Run

```
mog -m oracle-tables-to-snowflake <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```

  CREATE TABLE "SYSTEM"."ORA2SF_ORDERS" 
   (	"ORDER_ID" NUMBER(38,0) GENERATED ALWAYS AS IDENTITY MINVALUE 1 MAXVALUE 9999999999999999999999999999 INCREMENT BY 1 START WITH 1000 CACHE 20 NOORDER  NOCYCLE  NOKEEP  NOSCALE  NOT NULL ENABLE, 
	"CUSTOMER_ID" NUMBER(38,0) NOT NULL ENABLE, 
	"ORDER_NO" VARCHAR2(30) NOT NULL ENABLE, 
	"STATUS" VARCHAR2(20) DEFAULT 'PENDING' NOT NULL ENABLE, 
	"IS_ACTIVE" NUMBER(1,0) DEFAULT 1 NOT NULL ENABLE, 
	"QUANTITY" NUMBER(10,0) DEFAULT 0, 
	"UNIT_PRICE" NUMBER(12,2) NOT NULL ENABLE, 
	"DISCOUNT_PCT" NUMBER(5,4), 
	"TOTAL_AMOUNT" NUMBER(14,2) GENERATED ALWAYS AS ("QUANTITY"*"UNIT_PRICE") VIRTUAL , 
	"DESCRIPTION" CLOB, 
	"NOTES" VARCHAR2(4000), 
	"PAYLOAD" BLOB, 
	"CHECKSUM" RAW(16), 
	"CREATED_AT" DATE DEFAULT SYSDATE NOT NULL ENABLE, 
	"UPDATED_AT" TIMESTAMP (6), 
	"SHIP_TS" TIMESTAMP (6) WITH TIME ZONE, 
```

_(... 7 more line(s))_

Output:

```
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
```

_(... 4 more line(s))_

## Steps

- `run_mog`: Remove Oracle GET_DDL boilerplate (leading whitespace, "SCHEMA". qualifier, double-quote identifier quoting, ENABLE, CONSTRAINT USING INDEX clause, TIMESTAMP (n) spacing). Runs first; keeps the CREATE TABLE body.
- `flag_matching`: Flag Oracle DATE columns: Oracle DATE stores date+time, so it is mapped below to a zoneless Snowflake timestamp rather than a date-only column. Flagged before the type map so the marker text is not itself rewritten.
- `flag_matching`: Flag Oracle LOB columns (CLOB/BLOB) which are downcast to bounded Snowflake types below; large-object streaming/locator semantics do not carry over.
- `flag_matching`: Flag the Oracle virtual column (GENERATED ALWAYS AS (expr) VIRTUAL) before it is rewritten to a Snowflake computed column; the expression must be deterministic and use only supported functions.
- `flag_matching`: Flag CHECK constraints: Snowflake parses CHECK constraints for compatibility but does not enforce them, so the invariant must be enforced elsewhere (ETL / dbt tests).
- `replace_regex`: Rewrite an Oracle GENERATED ALWAYS AS IDENTITY column (with its MINVALUE/MAXVALUE/INCREMENT BY/START WITH/CACHE/NOORDER storage attributes) into a Snowflake IDENTITY START n INCREMENT n clause, preserving the seed and step.
- `replace_regex`: Rewrite an Oracle virtual column 'GENERATED ALWAYS AS (expr) VIRTUAL' to the Snowflake computed-column form 'AS (expr)'. Snowflake has no VIRTUAL keyword; the column type is kept.
- `replace_regex`: SYSDATE -> CURRENT_TIMESTAMP() (used in DEFAULTs). Runs before the DATE type map (word-boundary keeps it clear of SYSDATE anyway).
- `replace_regex`: SYSTIMESTAMP -> CURRENT_TIMESTAMP().
- `replace_regex`: VARCHAR2(n) -> VARCHAR(n).
- `replace_regex`: CLOB -> VARCHAR (Snowflake max VARCHAR; no CLOB type).
- `replace_regex`: RAW(n) -> BINARY (drop the byte length; Snowflake BINARY defaults to the max).
- `replace_regex`: BLOB -> BINARY.
- `replace_regex`: TIMESTAMP(n) WITH LOCAL TIME ZONE -> TIMESTAMP_LTZ (drop precision; mapped before the WITH TIME ZONE and plain variants).
- `replace_regex`: TIMESTAMP(n) WITH TIME ZONE -> TIMESTAMP_TZ (drop precision).
- `replace_regex`: Plain TIMESTAMP(n) -> TIMESTAMP_NTZ(n), keeping the fractional-seconds precision.
- `replace_regex`: DATE -> TIMESTAMP_NTZ (Oracle DATE carries a time-of-day; a Snowflake DATE would truncate it). See the TODO(mog) flagged above.
- `replace_regex`: Tidy the layout the USING INDEX strip leaves behind: fold a lone ',' that lands on its own line back onto the preceding constraint's ')'.

## Tags

`sql` `oracle` `snowflake` `warehouse` `convert` `migration`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

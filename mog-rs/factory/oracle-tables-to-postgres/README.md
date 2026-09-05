# Oracle GET_DDL to Postgres

Convert an Oracle DBMS_METADATA.GET_DDL export to PostgreSQL DDL

Convert an Oracle DBMS_METADATA.GET_DDL export (storage suppressed) to PostgreSQL DDL. Strips the GET_DDL boilerplate and maps types: NUMBER(1,0)->SMALLINT, NUMBER(p,0)->BIGINT, NUMBER(p,s)->NUMERIC(p,s), VARCHAR2->VARCHAR, CLOB->TEXT, DATE->TIMESTAMP, TIMESTAMP(n)->TIMESTAMP. Execution-validated against a real PostgreSQL engine.

## Run

```
mog -m oracle-tables-to-postgres <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```

  CREATE TABLE "APP"."CUSTOMERS"
   (	"ID" NUMBER(38,0) NOT NULL ENABLE,
	"NAME" VARCHAR2(200),
	"EMAIL" VARCHAR2(255),
	"DESCRIPTION" CLOB,
	"BALANCE" NUMBER(18,2),
	"IS_ACTIVE" NUMBER(1,0),
	"CREATED_AT" DATE,
	"UPDATED_AT" TIMESTAMP (6),
	 CONSTRAINT "PK_CUSTOMERS" PRIMARY KEY ("ID")
  USING INDEX  ENABLE
   )
```

Output:

```
CREATE TABLE CUSTOMERS
   (	ID BIGINT NOT NULL,
	NAME VARCHAR(200),
	EMAIL VARCHAR(255),
	DESCRIPTION TEXT,
	BALANCE NUMERIC(18,2),
	IS_ACTIVE SMALLINT,
	CREATED_AT TIMESTAMP,
	UPDATED_AT TIMESTAMP,
	 CONSTRAINT PK_CUSTOMERS PRIMARY KEY (ID)
  )
```

## Pipeline

- `run_mog`
- `replace_regex`
- `replace_regex`
- `replace_regex`
- `replace_regex`
- `replace_regex`
- `replace_regex`
- `replace_regex`
- `replace_regex`

## Tags

`sql` `oracle` `postgres` `convert` `migration`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

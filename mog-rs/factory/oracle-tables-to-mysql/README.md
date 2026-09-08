# Oracle GET_DDL to MySQL

Convert an Oracle DBMS_METADATA.GET_DDL export to MySQL DDL

Convert an Oracle DBMS_METADATA.GET_DDL export (storage suppressed) to MySQL DDL. Strips the GET_DDL boilerplate and maps types: NUMBER(1,0)->TINYINT(1), NUMBER(p,0)->BIGINT, NUMBER(p,s)->DECIMAL(p,s), VARCHAR2->VARCHAR, CLOB->LONGTEXT, DATE->DATETIME, TIMESTAMP(n)->DATETIME(n). Execution-validated against a real MySQL engine.

## Run

```
mog -m oracle-tables-to-mysql <file>
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
	DESCRIPTION LONGTEXT,
	BALANCE DECIMAL(18,2),
	IS_ACTIVE TINYINT(1),
	CREATED_AT DATETIME,
	UPDATED_AT DATETIME(6),
	 CONSTRAINT PK_CUSTOMERS PRIMARY KEY (ID)
  )
```

## Steps

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

`sql` `oracle` `mysql` `convert` `migration`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

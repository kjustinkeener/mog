# COBOL copybook to field spec CSV (lossy)

Extract a COBOL copybook field spec to CSV

Lossy parse of a COBOL copybook (the schema, i.e. the PIC clauses) into a CSV field spec: level,name,pic,type. Converts the copybook itself, not copybook-applied data (splitting fixed-width records needs the copybook as a second input, out of scope for a single-input tool). Keeps only elementary lines with a PIC clause and emits the level number, field name, raw PIC string, and a coarse type guess (text if the PIC has X/A, decimal if it has V or COMP-3, else integer). GROUP items (levels with no PIC) are dropped. Does not compute byte lengths, and does not resolve REDEFINES, OCCURS/OCCURS DEPENDING, 88-level condition names, or continuation. Good for a quick field inventory of a copybook.

## Run

```
mog -m cobol-copybook-to-fields <file>
```

Preview with `--dry-run` or `--diff`; write in place with `--in-place --backup .bak`.

## Example

Input:

```
      *****************************************************
      * CUSTOMER RECORD COPYBOOK
      *****************************************************
       01  CUSTOMER-RECORD.
           05  CUST-ID              PIC 9(5).
           05  CUST-NAME            PIC X(30).
           05  CUST-BALANCE         PIC S9(7)V99 COMP-3.
           05  CUST-STATUS          PIC A.
           05  CUST-ADDRESS.
               10  CUST-STREET      PIC X(40).
               10  CUST-ZIP         PIC 9(9).
           05  CUST-FLAGS           PIC X(4).
```

Output:

```
level,name,pic,type
05,CUST-ID,9(5),integer
05,CUST-NAME,X(30),text
05,CUST-BALANCE,S9(7)V99 COMP-3,decimal
05,CUST-STATUS,A,text
10,CUST-STREET,X(40),text
10,CUST-ZIP,9(9),integer
05,CUST-FLAGS,X(4),text
```

## Steps

- `keep_lines_matching`: Only lines with a PIC clause (drops group items and comments)
- `replace_regex_multiline`: level NAME PIC clause. -> level,NAME,pic
- `replace_regex_multiline`: text: PIC contains X or A
- `replace_regex_multiline`: decimal: PIC has V or COMP-3
- `replace_regex_multiline`: integer: remaining numeric PIC
- `prepend`: CSV header row

## Tags

`mainframe` `csv` `schema` `convert` `lossy` `best-effort`

---
<!-- Generated from the .mog by `mog market gen-docs`. Edit the .mog (or GUIDE.md), not this file. -->

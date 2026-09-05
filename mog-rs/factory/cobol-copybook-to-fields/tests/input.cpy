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

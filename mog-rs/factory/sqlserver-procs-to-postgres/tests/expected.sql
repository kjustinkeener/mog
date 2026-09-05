-- Database: lab

CREATE OR REPLACE PROCEDURE p_procdemo(
    customer varchar(50),
    amount numeric(19,4),
    note varchar(200) DEFAULT NULL
)
LANGUAGE plpgsql
AS $$
BEGIN
    -- TODO(mog): T-SQL cursor not portable to PL/pgSQL; rewrite as a FOR loop or refcursor: DECLARE cur CURSOR;

    INSERT INTO procdemo_orders (customer, amount, created)
    VALUES (customer, COALESCE(amount, 0), NOW());

    UPDATE procdemo_orders
    SET amount = amount
    WHERE customer = customer;

    RAISE NOTICE '%', 'inserted order for ' + customer;

    SELECT id, customer, amount, created
    FROM procdemo_orders
    WHERE customer = customer;
END;
$$;

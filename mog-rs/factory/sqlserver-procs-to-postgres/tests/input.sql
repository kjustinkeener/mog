USE [lab]
GO
/****** Object:  StoredProcedure [dbo].[p_procdemo]    Script Date: 8/29/2026 12:00:00 PM ******/
SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO

CREATE PROCEDURE [dbo].[p_procdemo]
    @customer NVARCHAR(50),
    @amount MONEY,
    @note NVARCHAR(200) = NULL
AS
BEGIN
    SET NOCOUNT ON;

    DECLARE @cur CURSOR;

    INSERT INTO dbo.procdemo_orders (customer, amount, created)
    VALUES (@customer, ISNULL(@amount, 0), GETDATE());

    UPDATE dbo.procdemo_orders
    SET amount = @amount
    WHERE customer = @customer;

    PRINT 'inserted order for ' + @customer;

    SELECT id, customer, amount, created
    FROM dbo.procdemo_orders
    WHERE customer = @customer;
END

GO

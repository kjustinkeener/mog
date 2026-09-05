USE [lab]
GO
/****** Object:  View [dbo].[v_viewdemo]    Script Date: Sat 8 29 2026  2:09:17 PM ******/
SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO
CREATE VIEW [dbo].[v_viewdemo]
WITH SCHEMABINDING
AS
SELECT TOP 100
    [order_id],
    [customer_name],
    ISNULL([region], 'UNKNOWN') AS region_clean,
    [quantity] * [unit_price] AS gross_amount,
    [quantity] * [unit_price] * (1 - ISNULL([discount], 0)) AS net_amount,
    LEN([customer_name]) AS name_len,
    GETDATE() AS snapshot_at
FROM [dbo].[viewdemo_orders]
WHERE [status_code] = 1 AND [quantity] > 0
ORDER BY [order_date] DESC
GO

USE [lab]
GO
/****** Object:  Table [dbo].[orders]    Script Date: Sat 8 29 2026  1:51:37 PM ******/
SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO
CREATE TABLE [dbo].[orders](
	[order_id] [bigint] IDENTITY(1,1) NOT NULL,
	[order_uid] [uniqueidentifier] NOT NULL,
	[customer_code] [nvarchar](50) NOT NULL,
	[description] [nvarchar](max) NULL,
	[status_flag] [bit] NOT NULL,
	[quantity] [int] NOT NULL,
	[unit_price] [money] NOT NULL,
	[discount_pct] [decimal](5, 2) NOT NULL,
	[line_total]  AS ([quantity]*[unit_price]) PERSISTED,
	[notes] [varchar](max) NULL,
	[payload] [varbinary](max) NULL,
	[created_at] [datetime2](7) NOT NULL,
	[updated_at] [datetimeoffset](7) NULL,
	[legacy_ts] [datetime] NULL
) ON [PRIMARY] TEXTIMAGE_ON [PRIMARY]
GO
/****** Object:  Index [PK_orders]    Script Date: Sat 8 29 2026  1:51:37 PM ******/
ALTER TABLE [dbo].[orders] ADD  CONSTRAINT [PK_orders] PRIMARY KEY CLUSTERED 
(
	[order_id] ASC
)WITH (PAD_INDEX = OFF, STATISTICS_NORECOMPUTE = OFF, SORT_IN_TEMPDB = OFF, IGNORE_DUP_KEY = OFF, ONLINE = OFF, ALLOW_ROW_LOCKS = ON, ALLOW_PAGE_LOCKS = ON) ON [PRIMARY]
GO
SET ANSI_PADDING ON
GO
/****** Object:  Index [UQ_orders_code]    Script Date: Sat 8 29 2026  1:51:37 PM ******/
ALTER TABLE [dbo].[orders] ADD  CONSTRAINT [UQ_orders_code] UNIQUE NONCLUSTERED 
(
	[customer_code] ASC
)WITH (PAD_INDEX = OFF, STATISTICS_NORECOMPUTE = OFF, SORT_IN_TEMPDB = OFF, IGNORE_DUP_KEY = OFF, ONLINE = OFF, ALLOW_ROW_LOCKS = ON, ALLOW_PAGE_LOCKS = ON) ON [PRIMARY]
GO
SET ANSI_PADDING ON
GO
/****** Object:  Index [IX_orders_status]    Script Date: Sat 8 29 2026  1:51:37 PM ******/
CREATE NONCLUSTERED INDEX [IX_orders_status] ON [dbo].[orders]
(
	[status_flag] ASC
)
INCLUDE ( 	[customer_code]) WITH (PAD_INDEX = OFF, STATISTICS_NORECOMPUTE = OFF, SORT_IN_TEMPDB = OFF, DROP_EXISTING = OFF, ONLINE = OFF, ALLOW_ROW_LOCKS = ON, ALLOW_PAGE_LOCKS = ON) ON [PRIMARY]
GO
ALTER TABLE [dbo].[orders] ADD  CONSTRAINT [DF_orders_uid]  DEFAULT (newsequentialid()) FOR [order_uid]
GO
ALTER TABLE [dbo].[orders] ADD  CONSTRAINT [DF_orders_status]  DEFAULT ((0)) FOR [status_flag]
GO
ALTER TABLE [dbo].[orders] ADD  CONSTRAINT [DF_orders_qty]  DEFAULT ((1)) FOR [quantity]
GO
ALTER TABLE [dbo].[orders] ADD  CONSTRAINT [DF_orders_disc]  DEFAULT ((0.00)) FOR [discount_pct]
GO
ALTER TABLE [dbo].[orders] ADD  CONSTRAINT [DF_orders_created]  DEFAULT (sysdatetime()) FOR [created_at]
GO
ALTER TABLE [dbo].[orders]  WITH CHECK ADD  CONSTRAINT [CK_orders_qty] CHECK  (([quantity]>(0)))
GO
ALTER TABLE [dbo].[orders] CHECK CONSTRAINT [CK_orders_qty]
GO
EXEC sys.sp_addextendedproperty @name=N'MS_Description', @value=N'Business customer code' , @level0type=N'SCHEMA',@level0name=N'dbo', @level1type=N'TABLE',@level1name=N'orders', @level2type=N'COLUMN',@level2name=N'customer_code'
GO
EXEC sys.sp_addextendedproperty @name=N'MS_Description', @value=N'Customer orders fact table' , @level0type=N'SCHEMA',@level0name=N'dbo', @level1type=N'TABLE',@level1name=N'orders'
GO

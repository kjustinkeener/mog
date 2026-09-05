USE [lab]
GO
/****** Object:  Table [dbo].[ms2bq_orders]    Script Date: Sat 8 29 2026  12:24:56 PM ******/
SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO
CREATE TABLE [dbo].[ms2bq_orders](
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
/****** Object:  Index [PK_ms2bq_orders]    Script Date: Sat 8 29 2026  12:24:56 PM ******/
ALTER TABLE [dbo].[ms2bq_orders] ADD  CONSTRAINT [PK_ms2bq_orders] PRIMARY KEY CLUSTERED 
(
	[order_id] ASC
)WITH (PAD_INDEX = OFF, STATISTICS_NORECOMPUTE = OFF, SORT_IN_TEMPDB = OFF, IGNORE_DUP_KEY = OFF, ONLINE = OFF, ALLOW_ROW_LOCKS = ON, ALLOW_PAGE_LOCKS = ON) ON [PRIMARY]
GO
SET ANSI_PADDING ON
GO
/****** Object:  Index [UQ_ms2bq_code]    Script Date: Sat 8 29 2026  12:24:56 PM ******/
ALTER TABLE [dbo].[ms2bq_orders] ADD  CONSTRAINT [UQ_ms2bq_code] UNIQUE NONCLUSTERED 
(
	[customer_code] ASC
)WITH (PAD_INDEX = OFF, STATISTICS_NORECOMPUTE = OFF, SORT_IN_TEMPDB = OFF, IGNORE_DUP_KEY = OFF, ONLINE = OFF, ALLOW_ROW_LOCKS = ON, ALLOW_PAGE_LOCKS = ON) ON [PRIMARY]
GO
SET ANSI_PADDING ON
GO
/****** Object:  Index [IX_ms2bq_status]    Script Date: Sat 8 29 2026  12:24:56 PM ******/
CREATE NONCLUSTERED INDEX [IX_ms2bq_status] ON [dbo].[ms2bq_orders]
(
	[status_flag] ASC
)
INCLUDE ( 	[customer_code]) WITH (PAD_INDEX = OFF, STATISTICS_NORECOMPUTE = OFF, SORT_IN_TEMPDB = OFF, DROP_EXISTING = OFF, ONLINE = OFF, ALLOW_ROW_LOCKS = ON, ALLOW_PAGE_LOCKS = ON) ON [PRIMARY]
GO
ALTER TABLE [dbo].[ms2bq_orders] ADD  CONSTRAINT [DF_ms2bq_uid]  DEFAULT (newsequentialid()) FOR [order_uid]
GO
ALTER TABLE [dbo].[ms2bq_orders] ADD  CONSTRAINT [DF_ms2bq_status]  DEFAULT ((0)) FOR [status_flag]
GO
ALTER TABLE [dbo].[ms2bq_orders] ADD  CONSTRAINT [DF_ms2bq_qty]  DEFAULT ((1)) FOR [quantity]
GO
ALTER TABLE [dbo].[ms2bq_orders] ADD  CONSTRAINT [DF_ms2bq_disc]  DEFAULT ((0.00)) FOR [discount_pct]
GO
ALTER TABLE [dbo].[ms2bq_orders] ADD  CONSTRAINT [DF_ms2bq_created]  DEFAULT (sysdatetime()) FOR [created_at]
GO
ALTER TABLE [dbo].[ms2bq_orders]  WITH CHECK ADD  CONSTRAINT [CK_ms2bq_qty] CHECK  (([quantity]>(0)))
GO
ALTER TABLE [dbo].[ms2bq_orders] CHECK CONSTRAINT [CK_ms2bq_qty]
GO
EXEC sys.sp_addextendedproperty @name=N'MS_Description', @value=N'Business customer code' , @level0type=N'SCHEMA',@level0name=N'dbo', @level1type=N'TABLE',@level1name=N'ms2bq_orders', @level2type=N'COLUMN',@level2name=N'customer_code'
GO
EXEC sys.sp_addextendedproperty @name=N'MS_Description', @value=N'Customer orders fact table' , @level0type=N'SCHEMA',@level0name=N'dbo', @level1type=N'TABLE',@level1name=N'ms2bq_orders'
GO

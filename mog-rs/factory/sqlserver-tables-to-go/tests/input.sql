USE [lab]
GO
/****** Object:  Table [dbo].[csgen_orders]    Script Date: Sat 8 29 2026  9:18:25 PM ******/
SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO
CREATE TABLE [dbo].[csgen_orders](
	[order_id] [bigint] IDENTITY(1,1) NOT NULL,
	[quantity] [int] NOT NULL,
	[small_count] [smallint] NOT NULL,
	[tiny_flag] [tinyint] NOT NULL,
	[customer_code] [nvarchar](50) NOT NULL,
	[description] [nvarchar](max) NULL,
	[legacy_note] [varchar](200) NULL,
	[fixed_code] [char](10) NULL,
	[region_code] [nchar](2) NULL,
	[long_text] [text] NULL,
	[is_active] [bit] NOT NULL,
	[discount_pct] [decimal](5, 2) NOT NULL,
	[unit_price] [money] NOT NULL,
	[ratio] [float] NOT NULL,
	[small_ratio] [real] NULL,
	[created_at] [datetime2](7) NOT NULL,
	[updated_at] [datetimeoffset](7) NULL,
	[legacy_ts] [datetime] NULL,
	[order_date] [date] NOT NULL,
	[order_time] [time](7) NULL,
	[order_uid] [uniqueidentifier] NOT NULL,
	[payload] [varbinary](max) NULL,
	[extra_props] [sql_variant] NULL
) ON [PRIMARY] TEXTIMAGE_ON [PRIMARY]
GO
/****** Object:  Index [PK_csgen_orders]    Script Date: Sat 8 29 2026  9:18:26 PM ******/
ALTER TABLE [dbo].[csgen_orders] ADD  CONSTRAINT [PK_csgen_orders] PRIMARY KEY CLUSTERED 
(
	[order_id] ASC
)WITH (PAD_INDEX = OFF, STATISTICS_NORECOMPUTE = OFF, SORT_IN_TEMPDB = OFF, IGNORE_DUP_KEY = OFF, ONLINE = OFF, ALLOW_ROW_LOCKS = ON, ALLOW_PAGE_LOCKS = ON) ON [PRIMARY]
GO
ALTER TABLE [dbo].[csgen_orders] ADD  CONSTRAINT [DF_csgen_active]  DEFAULT ((1)) FOR [is_active]
GO
ALTER TABLE [dbo].[csgen_orders] ADD  CONSTRAINT [DF_csgen_uid]  DEFAULT (newid()) FOR [order_uid]
GO

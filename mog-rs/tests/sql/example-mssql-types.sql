USE [ShopDemo]
GO
/****** Object:  Table [dbo].[TypeZoo]    Script Date: 1/2/2024 3:04:05 PM ******/
SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO
CREATE TABLE [dbo].[TypeZoo](
	[Id] [bigint] IDENTITY(1,1) NOT NULL,
	[Name] [nvarchar](200) NOT NULL,
	[Code] [nchar](10) NULL,
	[Notes] [nvarchar](max) NULL,
	[Body] [ntext] NULL,
	[Price] [money] NOT NULL,
	[Discount] [smallmoney] NULL,
	[Weight] [float] NULL,
	[Photo] [image] NULL,
	[Hash] [varbinary](max) NULL,
	[Flags] [binary](8) NULL,
	[CreatedAt] [datetime2](7) NOT NULL,
	[UpdatedAt] [datetimeoffset](7) NULL,
	[LegacyAt] [smalldatetime] NULL,
 CONSTRAINT [PK_TypeZoo] PRIMARY KEY CLUSTERED
(
	[Id] ASC
)WITH (PAD_INDEX = OFF, STATISTICS_NORECOMPUTE = OFF, IGNORE_DUP_KEY = OFF, ALLOW_ROW_LOCKS = ON, ALLOW_PAGE_LOCKS = ON, OPTIMIZE_FOR_SEQUENTIAL_KEY = OFF) ON [PRIMARY]
) ON [PRIMARY]
GO
ALTER TABLE [dbo].[TypeZoo] ADD  CONSTRAINT [DF_TypeZoo_CreatedAt]  DEFAULT (getutcdate()) FOR [CreatedAt]
GO

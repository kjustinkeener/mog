/****** Object:  Table [dbo].[Feature]    Script Date: 1/2/2024 3:04:05 PM ******/
SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO
CREATE TABLE [dbo].[Feature](
	[FeatureID] [bigint] IDENTITY(1000,5) NOT FOR REPLICATION NOT NULL,
	[FeatureGuid] [uniqueidentifier] ROWGUIDCOL NOT NULL,
	[Nickname] [nvarchar](100) SPARSE NULL,
	[SortKey] [bigint] NOT NULL,
	[Total] [money] NOT NULL,
	[TotalWithTax] AS ([Total] * 1.1) PERSISTED,
 CONSTRAINT [PK_Feature] PRIMARY KEY CLUSTERED
(
	[FeatureID] ASC
)WITH (PAD_INDEX = OFF, STATISTICS_NORECOMPUTE = OFF, IGNORE_DUP_KEY = OFF, ALLOW_ROW_LOCKS = ON, ALLOW_PAGE_LOCKS = ON, OPTIMIZE_FOR_SEQUENTIAL_KEY = OFF) ON [PRIMARY]
) ON [PRIMARY]
GO
ALTER TABLE [dbo].[Feature] ADD  CONSTRAINT [DF_Feature_Guid]  DEFAULT (newsequentialid()) FOR [FeatureGuid]
GO
ALTER TABLE [dbo].[Feature] ADD  CONSTRAINT [DF_Feature_Seq]  DEFAULT (NEXT VALUE FOR [dbo].[FeatureSeq]) FOR [SortKey]
GO

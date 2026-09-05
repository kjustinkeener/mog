USE [MogConvertTest]
GO
EXEC sys.sp_dropextendedproperty @name=N'MS_Description' , @level0type=N'SCHEMA',@level0name=N'dbo', @level1type=N'TABLE',@level1name=N'Widget'
GO
IF  EXISTS (SELECT * FROM sys.objects WHERE object_id = OBJECT_ID(N'[dbo].[Widget]') AND type in (N'U'))
DROP TABLE [dbo].[Widget]
GO
DROP INDEX [IX_Widget_Name] ON [dbo].[Widget]
GO
ALTER TABLE [dbo].[Widget] DROP CONSTRAINT [DF_Widget_Active]
GO
/****** Object:  Database [MogConvertTest]    Script Date: 1/2/2024 3:04:05 PM ******/
DROP DATABASE [MogConvertTest]
GO
CREATE DATABASE [MogConvertTest]
 CONTAINMENT = NONE
 ON  PRIMARY
( NAME = N'MogConvertTest', FILENAME = N'C:\data\MogConvertTest.mdf' )
 LOG ON
( NAME = N'MogConvertTest_log', FILENAME = N'C:\data\MogConvertTest_log.ldf' )
GO
ALTER DATABASE [MogConvertTest] SET COMPATIBILITY_LEVEL = 170
GO
ALTER DATABASE [MogConvertTest] SET ANSI_NULLS OFF
GO
EXEC sys.sp_db_vardecimal_storage_format N'MogConvertTest', N'ON'
GO
/****** Object:  Table [dbo].[Widget]    Script Date: 1/2/2024 3:04:05 PM ******/
SET ANSI_NULLS ON
GO
SET QUOTED_IDENTIFIER ON
GO
CREATE TABLE [dbo].[Widget](
	[WidgetID] [int] IDENTITY(1,1) NOT NULL,
	[Name] [nvarchar](100) NOT NULL,
	[IsActive] [bit] NOT NULL,
 CONSTRAINT [PK_Widget] PRIMARY KEY CLUSTERED
(
	[WidgetID] ASC
)WITH (PAD_INDEX = OFF, STATISTICS_NORECOMPUTE = OFF, IGNORE_DUP_KEY = OFF, ALLOW_ROW_LOCKS = ON, ALLOW_PAGE_LOCKS = ON, OPTIMIZE_FOR_SEQUENTIAL_KEY = OFF) ON [PRIMARY]
) ON [PRIMARY]
GO
SET ANSI_PADDING ON
GO
CREATE NONCLUSTERED INDEX [IX_Widget_Name] ON [dbo].[Widget]
(
	[Name] ASC
)WITH (PAD_INDEX = OFF) ON [PRIMARY]
GO
ALTER TABLE [dbo].[Widget]  WITH CHECK ADD  CONSTRAINT [CK_Widget_Name] CHECK  ((len([Name])>(0)))
GO
ALTER TABLE [dbo].[Widget] CHECK CONSTRAINT [CK_Widget_Name]
GO
EXEC sys.sp_addextendedproperty @name=N'MS_Description', @value=N'A widget.' , @level0type=N'SCHEMA',@level0name=N'dbo', @level1type=N'TABLE',@level1name=N'Widget'
GO

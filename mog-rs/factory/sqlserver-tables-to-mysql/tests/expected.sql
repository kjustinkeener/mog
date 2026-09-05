

CREATE TABLE Customers(
	CustomerID int AUTO_INCREMENT PRIMARY KEY,
	Name varchar(200) NOT NULL,
	Email varchar(255) NULL,
	Notes TEXT NULL,
	Balance decimal(18, 2) NULL,
	IsActive tinyint(1) NOT NULL,
	CreatedAt datetime NOT NULL,
	RowGuid char(36) NOT NULL
)

-- TODO(mog): primary key folded into CREATE TABLE

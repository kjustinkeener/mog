public class Actor
{
    public int ActorId { get; set; }
    public string FirstName { get; set; }
    public string LastName { get; set; }
    public DateTime LastUpdate { get; set; }
}

public class Film
{
    public int FilmId { get; set; }
    public string Title { get; set; }
    public string? Description { get; set; }
    public int? ReleaseYear { get; set; }
    public int LanguageId { get; set; }
    public int? OriginalLanguageId { get; set; }
    public sbyte RentalDuration { get; set; }
    public decimal RentalRate { get; set; }
    public short? Length { get; set; }
    public decimal ReplacementCost { get; set; }
    public string? Rating { get; set; } // TODO(mog): MySQL enum; values lost, consider a C# enum instead of string.
    public string? SpecialFeatures { get; set; } // TODO(mog): MySQL set; multi-value set lost, stored as string.
    public DateTime LastUpdate { get; set; }
}

public class Staff
{
    public int StaffId { get; set; }
    public string FirstName { get; set; }
    public string LastName { get; set; }
    public int AddressId { get; set; }
    public byte[]? Picture { get; set; }
    public string? Email { get; set; }
    public int StoreId { get; set; }
    public bool Active { get; set; }
    public string Username { get; set; }
    public string? Password { get; set; }
    public DateTime LastUpdate { get; set; }
}

public class Customer
{
    public int CustomerId { get; set; }
    public int StoreId { get; set; }
    public string FirstName { get; set; }
    public string LastName { get; set; }
    public string? Email { get; set; }
    public int AddressId { get; set; }
    public bool Active { get; set; }
    public DateTime CreateDate { get; set; }
    public DateTime? LastUpdate { get; set; }
}

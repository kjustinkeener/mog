public class Actor
{
    public int ActorId { get; set; }
    public string FirstName { get; set; }
    public string LastName { get; set; }
    public DateTime LastUpdate { get; set; }
}

public class Staff
{
    public int StaffId { get; set; }
    public string FirstName { get; set; }
    public string LastName { get; set; }
    public int AddressId { get; set; }
    public string? Email { get; set; }
    public int StoreId { get; set; }
    public bool Active { get; set; }
    public string Username { get; set; }
    public string? Password { get; set; }
    public DateTime LastUpdate { get; set; }
    public byte[]? Picture { get; set; }
}

public class Widget
{
    public long WidgetId { get; set; }
    public short SmallCount { get; set; }
    public long? BigCount { get; set; }
    public decimal UnitPrice { get; set; }
    public float? Ratio { get; set; }
    public double? PreciseRatio { get; set; }
    public string Sku { get; set; }
    public string? Grade { get; set; }
    public string? Description { get; set; }
    public bool IsActive { get; set; }
    public DateOnly CreatedOn { get; set; }
    public TimeOnly? OpenTime { get; set; }
    public DateTime MadeAt { get; set; }
    public DateTime UpdatedAt { get; set; }
    public Guid ExtId { get; set; }
    public string? Payload { get; set; }
    public string? Meta { get; set; }
    public byte[]? Thumbnail { get; set; }
    public string? Rating { get; set; } // TODO(mog): Postgres enum; consider a C# enum instead of string.
    public string? Fulltext { get; set; } // TODO(mog): Postgres full-search vector; stored as string, not searchable in C#.
    public string[]? SpecialFeatures { get; set; } // TODO(mog): Postgres array; verify element mapping and chosen C# collection.
}

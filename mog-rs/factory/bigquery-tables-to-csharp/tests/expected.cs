using System;
using System.Collections.Generic;

public class BqcsActor
{
    public long ActorId { get; set; }
    public string FirstName { get; set; }
    public string LastName { get; set; }
    public DateTime LastUpdate { get; set; }
}

public class BqcsStaff
{
    public long StaffId { get; set; }
    public string FirstName { get; set; }
    public string LastName { get; set; }
    public long AddressId { get; set; }
    public string? Email { get; set; }
    public long StoreId { get; set; }
    public bool Active { get; set; }
    public string Username { get; set; }
    public string? Password { get; set; }
    public DateTime LastUpdate { get; set; }
    public byte[]? Picture { get; set; }
}

public class BqcsWidget
{
    public long WidgetId { get; set; }
    public long SmallCount { get; set; }
    public long? BigCount { get; set; }
    public decimal UnitPrice { get; set; }
    public decimal? HugeTotal { get; set; } // TODO(mog): BigQuery 76-digit exact numbers overflow the built-in 28-29 digit exact-number type; use a big-number library if exactness matters.
    public double? Ratio { get; set; }
    public string Sku { get; set; }
    public string? Grade { get; set; }
    public string? Description { get; set; }
    public bool IsActive { get; set; }
    public DateOnly CreatedOn { get; set; }
    public TimeOnly? OpenTime { get; set; }
    public DateTime MadeAt { get; set; }
    public DateTime UpdatedAt { get; set; }
    public byte[]? Thumbnail { get; set; }
    public string? Payload { get; set; } // TODO(mog): BigQuery semi-structured column; carried in raw form -- deserialize it where you need field access.
    public Dictionary<string,object>? Shipping { get; set; } // TODO(mog): BigQuery record type; flattened to a dictionary -- declare a nested class for typed field access.
    public List<string> Tags { get; set; } // TODO(mog): BigQuery repeated column; never null (an absent value is the empty list) -- verify the element mapping.
    public List<double> Scores { get; set; } // TODO(mog): BigQuery repeated column; never null (an absent value is the empty list) -- verify the element mapping.
}

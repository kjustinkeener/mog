public class CsgenOrders
{
    public long OrderId { get; set; }
    public int Quantity { get; set; }
    public short SmallCount { get; set; }
    public byte TinyFlag { get; set; }
    public string CustomerCode { get; set; }
    public string? Description { get; set; }
    public string? LegacyNote { get; set; }
    public string? FixedCode { get; set; }
    public string? RegionCode { get; set; }
    public string? LongText { get; set; }
    public bool IsActive { get; set; }
    public decimal DiscountPct { get; set; }
    public decimal UnitPrice { get; set; }
    public double Ratio { get; set; }
    public float? SmallRatio { get; set; }
    public DateTime CreatedAt { get; set; }
    public DateTimeOffset? UpdatedAt { get; set; }
    public DateTime? LegacyTs { get; set; }
    public DateOnly OrderDate { get; set; }
    public TimeOnly? OrderTime { get; set; }
    public Guid OrderUid { get; set; }
    public byte[]? Payload { get; set; }
    public string? ExtraProps { get; set; } // TODO(mog): SQL Server variant type has no fixed C# type; stored as string.
}

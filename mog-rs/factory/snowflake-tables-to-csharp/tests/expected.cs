using System;

public class Invoice
{
    public long InvoiceId { get; set; } // TODO(mog): Snowflake column can exceed the signed 64-bit range; use System.Numerics.BigInteger if values can pass 9.2e18.
    public string FirstName { get; set; }
    public string LastName { get; set; }
    public DateTime UpdatedAt { get; set; }
}

public class PolicyHolder
{
    public long PolicyHolderId { get; set; } // TODO(mog): Snowflake column can exceed the signed 64-bit range; use System.Numerics.BigInteger if values can pass 9.2e18.
    public string AccountCode { get; set; }
    public string? Email { get; set; }
    public bool IsActive { get; set; }
    public DateOnly SignedUpOn { get; set; }
    public string? Notes { get; set; }
    public decimal Balance { get; set; }
    public DateTimeOffset? LastSeenAt { get; set; }
}

public class LedgerEntry
{
    public long EventId { get; set; } // TODO(mog): Snowflake column can exceed the signed 64-bit range; use System.Numerics.BigInteger if values can pass 9.2e18.
    public int SeqNo { get; set; }
    public int? RetryCount { get; set; }
    public decimal Amount { get; set; }
    public double? Rate { get; set; }
    public bool IsSampled { get; set; }
    public DateOnly EventDate { get; set; }
    public DateTime IngestedAt { get; set; }
    public DateTimeOffset OccurredAt { get; set; }
    public DateTimeOffset? SyncedAt { get; set; }
    public string SourceName { get; set; }
    public string? RawBody { get; set; }
    public string? Payload { get; set; } // TODO(mog): Snowflake semi-structured column; carried as raw JSON -- deserialize it if you know the shape.
    public string? Tags { get; set; } // TODO(mog): Snowflake list column has no declared element type; deserialize it into a typed collection yourself.
    public string? Context { get; set; } // TODO(mog): Snowflake semi-structured column; carried as raw JSON -- deserialize it if you know the shape.
    public byte[]? Checksum { get; set; }
}

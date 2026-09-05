type Shipment struct {
	ShipmentId int64 `db:"SHIPMENT_ID"` // TODO(mog): Snowflake column can exceed the signed 64-bit range; use math/big if values can pass 9.2e18.
	FirstName string `db:"FIRST_NAME"`
	LastName string `db:"LAST_NAME"`
	UpdatedAt time.Time `db:"UPDATED_AT"`
}

type SupplierAccount struct {
	SupplierAccountId int64 `db:"SUPPLIER_ACCOUNT_ID"` // TODO(mog): Snowflake column can exceed the signed 64-bit range; use math/big if values can pass 9.2e18.
	AccountCode string `db:"ACCOUNT_CODE"`
	Email *string `db:"EMAIL"`
	IsActive bool `db:"IS_ACTIVE"`
	SignedUpOn time.Time `db:"SIGNED_UP_ON"`
	Notes *string `db:"NOTES"`
	Balance string `db:"BALANCE"` // TODO(mog): Snowflake fixed-point column; Go has no native exact-scale kind -- parse with an exact-arithmetic library.
	LastSeenAt *time.Time `db:"LAST_SEEN_AT"`
}

type SensorReading struct {
	EventId int64 `db:"EVENT_ID"` // TODO(mog): Snowflake column can exceed the signed 64-bit range; use math/big if values can pass 9.2e18.
	SeqNo int32 `db:"SEQ_NO"`
	RetryCount *int32 `db:"RETRY_COUNT"`
	Amount string `db:"AMOUNT"` // TODO(mog): Snowflake fixed-point column; Go has no native exact-scale kind -- parse with an exact-arithmetic library.
	Rate *float64 `db:"RATE"`
	IsSampled bool `db:"IS_SAMPLED"`
	EventDate time.Time `db:"EVENT_DATE"`
	IngestedAt time.Time `db:"INGESTED_AT"`
	OccurredAt time.Time `db:"OCCURRED_AT"`
	SyncedAt *time.Time `db:"SYNCED_AT"`
	SourceName string `db:"SOURCE_NAME"`
	RawBody *string `db:"RAW_BODY"`
	Payload json.RawMessage `db:"PAYLOAD"` // TODO(mog): Snowflake semi-structured column; unmarshal it into a concrete struct if you know the shape.
	Tags json.RawMessage `db:"TAGS"` // TODO(mog): Snowflake list column has no declared element type; unmarshal it into a typed slice yourself.
	Context json.RawMessage `db:"CONTEXT"` // TODO(mog): Snowflake semi-structured column; unmarshal it into a concrete struct if you know the shape.
	Checksum []byte `db:"CHECKSUM"`
}

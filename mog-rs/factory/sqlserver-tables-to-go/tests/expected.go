type CsgenOrders struct {
	OrderId int64 `db:"order_id"`
	Quantity int32 `db:"quantity"`
	SmallCount int16 `db:"small_count"`
	TinyFlag int16 `db:"tiny_flag"`
	CustomerCode string `db:"customer_code"`
	Description *string `db:"description"`
	LegacyNote *string `db:"legacy_note"`
	FixedCode *string `db:"fixed_code"`
	RegionCode *string `db:"region_code"`
	LongText *string `db:"long_text"`
	IsActive bool `db:"is_active"`
	DiscountPct string `db:"discount_pct"` // TODO(mog): exact-precision or currency column; mapped to a Go string (no native fixed-point) -- parse with a fixed-point library for math.
	UnitPrice string `db:"unit_price"` // TODO(mog): exact-precision or currency column; mapped to a Go string (no native fixed-point) -- parse with a fixed-point library for math.
	Ratio float64 `db:"ratio"`
	SmallRatio *float32 `db:"small_ratio"`
	CreatedAt time.Time `db:"created_at"`
	UpdatedAt *time.Time `db:"updated_at"`
	LegacyTs *time.Time `db:"legacy_ts"`
	OrderDate time.Time `db:"order_date"`
	OrderTime *time.Time `db:"order_time"`
	OrderUid string `db:"order_uid"`
	Payload []byte `db:"payload"`
	ExtraProps *any `db:"extra_props"` // TODO(mog): SQL Server sql_variant; runtime-typed (Go any), verify mapping.
}

type Actor struct {
	ActorId int32 `db:"actor_id"`
	FirstName string `db:"first_name"`
	LastName string `db:"last_name"`
	LastUpdate time.Time `db:"last_update"`
}

type Staff struct {
	StaffId int32 `db:"staff_id"`
	FirstName string `db:"first_name"`
	LastName string `db:"last_name"`
	AddressId int32 `db:"address_id"`
	Email *string `db:"email"`
	StoreId int32 `db:"store_id"`
	Active bool `db:"active"`
	Username string `db:"username"`
	Password *string `db:"password"`
	LastUpdate time.Time `db:"last_update"`
	Picture []byte `db:"picture"`
}

type Widget struct {
	WidgetId int64 `db:"widget_id"`
	SmallCount int16 `db:"small_count"`
	BigCount *int64 `db:"big_count"`
	UnitPrice string `db:"unit_price"` // TODO(mog): Postgres exact-precision column; mapped to a Go string (no native fixed-point) -- parse with a fixed-point library for arithmetic.
	Ratio *float32 `db:"ratio"`
	PreciseRatio *float64 `db:"precise_ratio"`
	Sku string `db:"sku"`
	Grade *string `db:"grade"`
	Description *string `db:"description"`
	IsActive bool `db:"is_active"`
	CreatedOn time.Time `db:"created_on"`
	OpenTime *time.Time `db:"open_time"`
	MadeAt time.Time `db:"made_at"`
	UpdatedAt time.Time `db:"updated_at"`
	ExtId string `db:"ext_id"`
	Payload json.RawMessage `db:"payload"`
	Meta json.RawMessage `db:"meta"`
	Thumbnail []byte `db:"thumbnail"`
	Rating *string `db:"rating"` // TODO(mog): Postgres enum; represented as string in Go (define a named type + constants if you need type safety).
	Fulltext *string `db:"fulltext"` // TODO(mog): Postgres full-search vector; stored as string, not searchable in Go.
	SpecialFeatures []string `db:"special_features"` // TODO(mog): Postgres array; mapped to a Go slice -- verify the element type.
}

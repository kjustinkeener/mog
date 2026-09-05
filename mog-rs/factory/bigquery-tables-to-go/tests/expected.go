type BqgoActor struct {
	ActorId int64 `bigquery:"actor_id"`
	FirstName string `bigquery:"first_name"`
	LastName string `bigquery:"last_name"`
	LastUpdate time.Time `bigquery:"last_update"`
}

type BqgoStaff struct {
	StaffId int64 `bigquery:"staff_id"`
	FirstName string `bigquery:"first_name"`
	LastName string `bigquery:"last_name"`
	AddressId int64 `bigquery:"address_id"`
	Email *string `bigquery:"email"`
	StoreId int64 `bigquery:"store_id"`
	Active bool `bigquery:"active"`
	Username string `bigquery:"username"`
	Password *string `bigquery:"password"`
	LastUpdate time.Time `bigquery:"last_update"`
	Picture []byte `bigquery:"picture"`
}

type BqgoWidget struct {
	WidgetId int64 `bigquery:"widget_id"`
	SmallCount int64 `bigquery:"small_count"`
	BigCount *int64 `bigquery:"big_count"`
	UnitPrice string `bigquery:"unit_price"` // TODO(mog): BigQuery exact-precision column; no native Go fixed-point -- carried as text, parse with a fixed-point library for arithmetic.
	HugeTotal *string `bigquery:"huge_total"` // TODO(mog): BigQuery 76-digit exact numbers have no native Go equivalent; carried as text -- use math/big or a fixed-point library for arithmetic.
	Ratio *float64 `bigquery:"ratio"`
	Sku string `bigquery:"sku"`
	Grade *string `bigquery:"grade"`
	Description *string `bigquery:"description"`
	IsActive bool `bigquery:"is_active"`
	CreatedOn time.Time `bigquery:"created_on"`
	OpenTime *time.Time `bigquery:"open_time"`
	MadeAt time.Time `bigquery:"made_at"`
	UpdatedAt time.Time `bigquery:"updated_at"`
	Thumbnail []byte `bigquery:"thumbnail"`
	Payload json.RawMessage `bigquery:"payload"` // TODO(mog): BigQuery semi-structured column; kept in its raw encoded form -- unmarshal into a concrete Go type where you need field access.
	Shipping map[string]any `bigquery:"shipping"` // TODO(mog): BigQuery record type; flattened to a Go map -- declare a nested Go struct for typed field access.
	Tags []string `bigquery:"tags"` // TODO(mog): BigQuery repeated column; never null (an absent value is the empty slice) -- verify the element type.
	Scores []float64 `bigquery:"scores"` // TODO(mog): BigQuery repeated column; never null (an absent value is the empty slice) -- verify the element type.
}

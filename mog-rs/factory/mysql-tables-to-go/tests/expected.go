type Actor struct {
	ActorId int32 `db:"actor_id"`
	FirstName string `db:"first_name"`
	LastName string `db:"last_name"`
	LastUpdate time.Time `db:"last_update"`
}

type Film struct {
	FilmId int32 `db:"film_id"`
	Title string `db:"title"`
	Description *string `db:"description"`
	ReleaseYear *int16 `db:"release_year"`
	LanguageId int32 `db:"language_id"`
	OriginalLanguageId *int32 `db:"original_language_id"`
	RentalDuration int8 `db:"rental_duration"`
	RentalRate string `db:"rental_rate"` // TODO(mog): MySQL exact-precision column; mapped to a Go string (no native fixed-point) -- parse with a fixed-point library for arithmetic.
	Length *int16 `db:"length"`
	ReplacementCost string `db:"replacement_cost"` // TODO(mog): MySQL exact-precision column; mapped to a Go string (no native fixed-point) -- parse with a fixed-point library for arithmetic.
	Rating *string `db:"rating"` // TODO(mog): MySQL enum; represented as a Go string (define named constants if you need type safety).
	SpecialFeatures *string `db:"special_features"` // TODO(mog): MySQL set; comma-joined values in one Go string -- split on ',' or model as a slice.
	LastUpdate time.Time `db:"last_update"`
}

type Staff struct {
	StaffId int32 `db:"staff_id"`
	FirstName string `db:"first_name"`
	LastName string `db:"last_name"`
	AddressId int32 `db:"address_id"`
	Picture []byte `db:"picture"`
	Email *string `db:"email"`
	StoreId int32 `db:"store_id"`
	Active bool `db:"active"`
	Username string `db:"username"`
	Password *string `db:"password"`
	LastUpdate time.Time `db:"last_update"`
}

type Customer struct {
	CustomerId int32 `db:"customer_id"`
	StoreId int32 `db:"store_id"`
	FirstName string `db:"first_name"`
	LastName string `db:"last_name"`
	Email *string `db:"email"`
	AddressId int32 `db:"address_id"`
	Active bool `db:"active"`
	CreateDate time.Time `db:"create_date"`
	LastUpdate *time.Time `db:"last_update"`
}

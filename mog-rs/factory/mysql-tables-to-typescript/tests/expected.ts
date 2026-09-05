export interface Actor {
  actor_id: number;
  first_name: string;
  last_name: string;
  last_update: string;
}

export interface Film {
  film_id: number;
  title: string;
  description: string | null;
  release_year: number | null;
  language_id: number;
  original_language_id: number | null;
  rental_duration: number;
  rental_rate: number;
  length: number | null;
  replacement_cost: number;
  rating: string | null; // TODO(mog): MySQL enum; consider a TS string-literal union.
  special_features: string | null; // TODO(mog): MySQL set; comma-joined values, consider string[].
  last_update: string;
}

export interface Staff {
  staff_id: number;
  first_name: string;
  last_name: string;
  address_id: number;
  picture: Uint8Array | null;
  email: string | null;
  store_id: number;
  active: boolean;
  username: string;
  password: string | null;
  last_update: string;
}

export interface Customer {
  customer_id: number;
  store_id: number;
  first_name: string;
  last_name: string;
  email: string | null;
  address_id: number;
  active: boolean;
  create_date: string;
  last_update: string | null;
}

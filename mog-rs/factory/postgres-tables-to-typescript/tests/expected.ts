export interface Actor {
  actor_id: number;
  first_name: string;
  last_name: string;
  last_update: string;
}

export interface Staff {
  staff_id: number;
  first_name: string;
  last_name: string;
  address_id: number;
  email: string | null;
  store_id: number;
  active: boolean;
  username: string;
  password: string | null;
  last_update: string;
  picture: Uint8Array | null;
}

export interface Widget {
  widget_id: number;
  small_count: number;
  big_count: number | null;
  unit_price: number;
  ratio: number | null;
  precise_ratio: number | null;
  sku: string;
  grade: string | null;
  description: string | null;
  is_active: boolean;
  created_on: string;
  open_time: string | null;
  made_at: string;
  updated_at: string;
  ext_id: string;
  payload: unknown | null;
  meta: unknown | null;
  thumbnail: Uint8Array | null;
  rating: string | null; // TODO(mog): Postgres enum; consider a TypeScript union type instead of string.
  fulltext: string | null; // TODO(mog): Postgres full-search vector; stored as string, not searchable in TypeScript.
  special_features: string[] | null; // TODO(mog): Postgres array; verify element mapping and chosen TypeScript type.
}

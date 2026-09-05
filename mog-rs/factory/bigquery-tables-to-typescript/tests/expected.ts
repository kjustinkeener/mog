export interface BqtsActor {
  actor_id: number;
  first_name: string;
  last_name: string;
  last_update: string;
}

export interface BqtsStaff {
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

export interface BqtsWidget {
  widget_id: number;
  small_count: number;
  big_count: number | null;
  unit_price: number;
  huge_total: number | null; // TODO(mog): BigQuery 76-digit exact numbers exceed JavaScript's IEEE-754 precision; carry the value as text if exactness matters.
  ratio: number | null;
  sku: string;
  grade: string | null;
  description: string | null;
  is_active: boolean;
  created_on: string;
  open_time: string | null;
  made_at: string;
  updated_at: string;
  thumbnail: Uint8Array | null;
  payload: unknown | null; // TODO(mog): BigQuery semi-structured column; typed as unknown -- narrow it with a parser or a hand-written interface.
  shipping: Record<string,unknown> | null; // TODO(mog): BigQuery record type; flattened to an index signature -- declare a nested interface for typed field access.
  tags: string[]; // TODO(mog): BigQuery repeated column; never null (an absent value is the empty array) -- verify the element mapping.
  scores: number[]; // TODO(mog): BigQuery repeated column; never null (an absent value is the empty array) -- verify the element mapping.
}

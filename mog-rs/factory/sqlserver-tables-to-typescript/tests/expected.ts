export interface CsgenOrders {
  order_id: number;
  quantity: number;
  small_count: number;
  tiny_flag: number;
  customer_code: string;
  description: string | null;
  legacy_note: string | null;
  fixed_code: string | null;
  region_code: string | null;
  long_text: string | null;
  is_active: boolean;
  discount_pct: number;
  unit_price: number;
  ratio: number;
  small_ratio: number | null;
  created_at: string;
  updated_at: string | null;
  legacy_ts: string | null;
  order_date: string;
  order_time: string | null;
  order_uid: string;
  payload: Uint8Array | null;
  extra_props: unknown | null; // TODO(mog): SQL Server sql_variant; runtime-typed, verify mapping.
}

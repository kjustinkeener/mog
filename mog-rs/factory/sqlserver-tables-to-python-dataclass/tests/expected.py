from __future__ import annotations
from dataclasses import dataclass
from datetime import date, datetime, time
from decimal import Decimal
from typing import Any
from uuid import UUID

@dataclass
class CsgenOrders:
    order_id: int
    quantity: int
    small_count: int
    tiny_flag: int
    customer_code: str
    description: str | None
    legacy_note: str | None
    fixed_code: str | None
    region_code: str | None
    long_text: str | None
    is_active: bool
    discount_pct: Decimal
    unit_price: Decimal
    ratio: float
    small_ratio: float | None
    created_at: datetime
    updated_at: datetime | None
    legacy_ts: datetime | None
    order_date: date
    order_time: time | None
    order_uid: UUID
    payload: bytes | None
    extra_props: Any | None # TODO(mog): SQL Server sql_variant; runtime-typed (typing.Any), verify mapping.

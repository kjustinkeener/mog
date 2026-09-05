from __future__ import annotations
from dataclasses import dataclass
from datetime import date, datetime, time
from decimal import Decimal
from typing import Any

@dataclass
class BqpyActor:
    actor_id: int
    first_name: str
    last_name: str
    last_update: datetime

@dataclass
class BqpyStaff:
    staff_id: int
    first_name: str
    last_name: str
    address_id: int
    email: str | None
    store_id: int
    active: bool
    username: str
    password: str | None
    last_update: datetime
    picture: bytes | None

@dataclass
class BqpyWidget:
    widget_id: int
    small_count: int
    big_count: int | None
    unit_price: Decimal
    huge_total: Decimal | None # TODO(mog): BigQuery 76-digit exact numbers exceed the default context precision of 28; raise getcontext().prec before arithmetic.
    ratio: float | None
    sku: str
    grade: str | None
    description: str | None
    is_active: bool
    created_on: date
    open_time: time | None
    made_at: datetime
    updated_at: datetime
    thumbnail: bytes | None
    payload: dict[str, Any] | None # TODO(mog): BigQuery semi-structured column; typed as dict (assumes object payloads) -- use a TypedDict for field-level checking.
    shipping: dict[str, Any] | None # TODO(mog): BigQuery record type; flattened to dict -- declare a nested dataclass or TypedDict for typed field access.
    tags: list[str] # TODO(mog): BigQuery repeated column; never null (an absent value is the empty list) -- verify the element mapping.
    scores: list[float] # TODO(mog): BigQuery repeated column; never null (an absent value is the empty list) -- verify the element mapping.

from __future__ import annotations
from dataclasses import dataclass
from datetime import date, datetime, time
from decimal import Decimal
from uuid import UUID

@dataclass
class Actor:
    actor_id: int
    first_name: str
    last_name: str
    last_update: datetime

@dataclass
class Staff:
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
class Widget:
    widget_id: int
    small_count: int
    big_count: int | None
    unit_price: Decimal
    ratio: float | None
    precise_ratio: float | None
    sku: str
    grade: str | None
    description: str | None
    is_active: bool
    created_on: date
    open_time: time | None
    made_at: datetime
    updated_at: datetime
    ext_id: UUID
    payload: dict | None
    meta: dict | None
    thumbnail: bytes | None
    rating: str | None # TODO(mog): Postgres enum; consider a Python Enum instead of a plain string.
    fulltext: str | None # TODO(mog): Postgres full-search vector; stored as str, not searchable in Python.
    special_features: list[str] | None # TODO(mog): Postgres array; verify element mapping and chosen Python collection.

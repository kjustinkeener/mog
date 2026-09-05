from __future__ import annotations
from dataclasses import dataclass
from datetime import date, datetime, time
from decimal import Decimal

@dataclass
class Actor:
    actor_id: int
    first_name: str
    last_name: str
    last_update: datetime

@dataclass
class Film:
    film_id: int
    title: str
    description: str | None
    release_year: int | None
    language_id: int
    original_language_id: int | None
    rental_duration: int
    rental_rate: Decimal
    length: int | None
    replacement_cost: Decimal
    rating: str | None # TODO(mog): MySQL enum; consider a Python Enum instead of a plain str.
    special_features: str | None # TODO(mog): MySQL set; comma-joined values in one str -- consider a set[str] or list[str].
    last_update: datetime

@dataclass
class Staff:
    staff_id: int
    first_name: str
    last_name: str
    address_id: int
    picture: bytes | None
    email: str | None
    store_id: int
    active: bool
    username: str
    password: str | None
    last_update: datetime

@dataclass
class Customer:
    customer_id: int
    store_id: int
    first_name: str
    last_name: str
    email: str | None
    address_id: int
    active: bool
    create_date: datetime
    last_update: datetime | None

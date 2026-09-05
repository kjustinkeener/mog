from __future__ import annotations
from dataclasses import dataclass
from datetime import date, datetime, time
from decimal import Decimal
from typing import Any

@dataclass
class Subscriber:
    subscriber_id: int
    first_name: str
    last_name: str
    updated_at: datetime

@dataclass
class CampaignRun:
    campaign_run_id: int
    account_code: str
    email: str | None
    is_active: bool
    signed_up_on: date
    notes: str | None
    balance: Decimal # TODO(mog): Snowflake fixed-point column; enable the connector's exact-value option so it does not approximate.
    last_seen_at: datetime | None

@dataclass
class PageView:
    event_id: int
    seq_no: int
    retry_count: int | None
    amount: Decimal # TODO(mog): Snowflake fixed-point column; enable the connector's exact-value option so it does not approximate.
    rate: float | None
    is_sampled: bool
    event_date: date
    ingested_at: datetime
    occurred_at: datetime
    synced_at: datetime | None
    source_name: str
    raw_body: str | None
    payload: dict[str, Any] | None # TODO(mog): Snowflake semi-structured column; the connector hands back raw JSON unless you parse it.
    tags: list[Any] | None # TODO(mog): Snowflake list column has no declared element type; narrow it yourself.
    context: dict[str, Any] | None # TODO(mog): Snowflake semi-structured column; the connector hands back raw JSON unless you parse it.
    checksum: bytes | None

from __future__ import annotations

from datetime import datetime
from typing import List, Optional

from pydantic import BaseModel, Field, validator


class RawSequence(BaseModel):
    """Representation of sequence + metadata prior to transformation."""

    identifier: str = Field(..., alias="id")
    name: Optional[str] = None
    lineage: Optional[str] = None
    location: Optional[str] = None
    collection_date: Optional[datetime] = None
    sequence: str

    @validator("sequence")
    def sequence_uppercase(cls, value: str) -> str:
        cleaned = value.strip().replace("\n", "").upper()
        if not cleaned:
            raise ValueError("Sequence must not be empty")
        return cleaned

    @validator("collection_date", pre=True)
    def parse_date(cls, value):
        if value in (None, "", "NA", "N/A"):
            return None
        if isinstance(value, datetime):
            return value
        for fmt in ("%Y-%m-%d", "%d.%m.%Y", "%Y/%m/%d"):
            try:
                return datetime.strptime(str(value), fmt)
            except ValueError:
                continue
        raise ValueError(f"Unsupported date format: {value}")


class VariantPayload(BaseModel):
    """Payload structure used when uploading variants to the backend API."""

    name: str
    lineage: str
    location: str
    sequence: str


class IngestionError(BaseModel):
    identifier: str
    message: str


class IngestionReport(BaseModel):
    total_records: int
    successful: int
    failed: int
    errors: List[IngestionError] = []

    @classmethod
    def from_results(
        cls, total: int, succeeded: List[str], failures: List[IngestionError]
    ) -> "IngestionReport":
        return cls(
            total_records=total,
            successful=len(succeeded),
            failed=len(failures),
            errors=failures,
        )



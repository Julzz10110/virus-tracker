from dataclasses import dataclass
from pathlib import Path
from typing import Optional
import os


def _bool_from_env(value: Optional[str], default: bool = False) -> bool:
    if value is None:
        return default
    return value.lower() in {"1", "true", "yes", "on"}


@dataclass
class BackendSettings:
    """Configuration required to communicate with the Rust backend API."""

    base_url: str = os.getenv("BACKEND_BASE_URL", "http://localhost:8080/api")
    timeout_seconds: float = float(os.getenv("BACKEND_TIMEOUT_SECONDS", "10"))
    update_reference: bool = _bool_from_env(os.getenv("UPDATE_REFERENCE"), False)


@dataclass
class IngestionSettings:
    """Configuration describing the ingestion data sources."""

    sequences_path: Path
    metadata_path: Optional[Path] = None
    chunk_size: int = int(os.getenv("INGESTION_CHUNK_SIZE", "50"))
    dry_run: bool = _bool_from_env(os.getenv("INGESTION_DRY_RUN"), False)

    def validate(self) -> None:
        if not self.sequences_path.exists():
            raise FileNotFoundError(f"Sequences file not found: {self.sequences_path}")
        if self.metadata_path and not self.metadata_path.exists():
            raise FileNotFoundError(
                f"Metadata file not found: {self.metadata_path}"
            )
        if self.chunk_size <= 0:
            raise ValueError("chunk_size must be a positive integer")


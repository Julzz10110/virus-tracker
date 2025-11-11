"""
Bio Processor package for virus tracker.

This package provides utilities for:
  * Loading genomic sequences and associated metadata from local files
  * Cleaning and transforming the data into structures expected by the backend
  * Interacting with the Rust backend API to upload variants and manage the reference sequence
  * Experimental analysis helpers exposed via FastAPI (see api module)
"""

from .config import BackendSettings, IngestionSettings
from .pipeline import run_ingestion_pipeline

__all__ = [
    "BackendSettings",
    "IngestionSettings",
    "run_ingestion_pipeline",
]



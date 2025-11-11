from __future__ import annotations

import argparse
import logging
from pathlib import Path

from .config import BackendSettings, IngestionSettings
from .pipeline import run_ingestion_pipeline
from .utils import load_reference_sequence

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(levelname)s - %(name)s - %(message)s",
)
logger = logging.getLogger(__name__)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Ingest genomic sequences into the virus tracker backend."
    )
    parser.add_argument(
        "--sequences",
        required=True,
        type=Path,
        help="Path to FASTA file containing sequences.",
    )
    parser.add_argument(
        "--metadata",
        type=Path,
        help="Optional CSV/TSV file containing per-sequence metadata.",
    )
    parser.add_argument(
        "--reference",
        type=Path,
        help="Optional FASTA file with a single reference sequence for alignment.",
    )
    parser.add_argument(
        "--base-url",
        type=str,
        default=None,
        help="Backend API base URL (overrides BACKEND_BASE_URL env).",
    )
    parser.add_argument(
        "--chunk-size",
        type=int,
        default=None,
        help="Number of variants to upload per batch.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Process data without uploading to backend.",
    )
    parser.add_argument(
        "--update-reference",
        action="store_true",
        help="Update backend reference sequence before uploading variants.",
    )
    return parser.parse_args()


def main() -> None:
    args = parse_args()

    backend = BackendSettings()
    if args.base_url:
        backend.base_url = args.base_url
    backend.update_reference = args.update_reference or backend.update_reference

    ingestion = IngestionSettings(
        sequences_path=args.sequences,
        metadata_path=args.metadata,
        dry_run=args.dry_run,
        chunk_size=args.chunk_size or 50,
    )

    reference_sequence: Optional[str] = None
    if args.reference:
        reference_sequence = load_reference_sequence(args.reference)

    report = run_ingestion_pipeline(
        ingestion=ingestion, backend=backend, reference_sequence=reference_sequence
    )

    logger.info(
        "Ingestion report: processed=%s, success=%s, failed=%s",
        report.total_records,
        report.successful,
        report.failed,
    )
    if report.failed:
        logger.info("Errors:")
        for error in report.errors:
            logger.info("  - %s: %s", error.identifier, error.message)


if __name__ == "__main__":
    main()


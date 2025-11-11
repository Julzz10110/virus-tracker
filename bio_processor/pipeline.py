from __future__ import annotations

from typing import List, Optional
import logging

from .backend_client import BackendClient
from .config import BackendSettings, IngestionSettings
from .loaders import (
    iter_in_chunks,
    load_metadata_from_csv,
    load_sequences_from_fasta,
    merge_metadata_with_sequences,
)
from .schemas import IngestionError, IngestionReport, RawSequence, VariantPayload

logger = logging.getLogger(__name__)


def _to_variant_payload(record: RawSequence) -> VariantPayload:
    return VariantPayload(
        name=record.name or record.identifier,
        lineage=record.lineage or "Unknown",
        location=record.location or "Unknown",
        sequence=record.sequence,
    )


def run_ingestion_pipeline(
    ingestion: IngestionSettings,
    backend: BackendSettings,
    reference_sequence: Optional[str] = None,
) -> IngestionReport:
    """
    Execute the ingestion pipeline:
      1. Load sequences and optional metadata.
      2. Merge and validate records.
      3. Optionally update backend reference sequence.
      4. Upload variants in batches.
    """

    logger.info("Validating ingestion configuration")
    ingestion.validate()

    logger.info("Loading sequences from %s", ingestion.sequences_path)
    sequences = load_sequences_from_fasta(ingestion.sequences_path)

    metadata_df = None
    if ingestion.metadata_path:
        logger.info("Loading metadata from %s", ingestion.metadata_path)
        metadata_df = load_metadata_from_csv(ingestion.metadata_path)

    logger.info("Merging metadata with sequences")
    records: List[RawSequence] = merge_metadata_with_sequences(sequences, metadata_df)

    client = BackendClient(
        base_url=backend.base_url,
        timeout_seconds=backend.timeout_seconds,
    )

    if backend.update_reference and reference_sequence:
        logger.info("Updating backend reference sequence")
        client.update_reference(reference_sequence)

    if ingestion.dry_run:
        logger.info("Dry-run enabled; skipping upload to backend")
        return IngestionReport.from_results(len(records), [], [])

    succeeded: List[str] = []
    failures: List[IngestionError] = []

    logger.info("Uploading %d variants in chunks of %d", len(records), ingestion.chunk_size)
    for chunk in iter_in_chunks(records, ingestion.chunk_size):
        for record in chunk:
            try:
                payload = _to_variant_payload(record)
                client.upload_variant(payload)
                succeeded.append(record.identifier)
            except Exception as exc:  # pragma: no cover - network/external errors
                logger.exception("Failed to upload variant %s", record.identifier)
                failures.append(
                    IngestionError(
                        identifier=record.identifier,
                        message=str(exc),
                    )
                )

    report = IngestionReport.from_results(len(records), succeeded, failures)
    logger.info(
        "Ingestion completed: %s successes, %s failures",
        report.successful,
        report.failed,
    )
    return report



from __future__ import annotations

from pathlib import Path
from typing import Optional

import logging
from Bio import SeqIO

logger = logging.getLogger(__name__)


def load_reference_sequence(path: Optional[Path]) -> Optional[str]:
    if not path:
        return None

    records = list(SeqIO.parse(str(path), "fasta"))
    if not records:
        raise ValueError(f"No sequences found in reference file: {path}")
    if len(records) > 1:
        logger.warning(
            "Reference file %s contains multiple sequences; using the first entry.",
            path,
        )
        return str(records[0].seq).upper()
    return str(records[0].seq).upper()


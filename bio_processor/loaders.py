from __future__ import annotations

from pathlib import Path
from typing import Dict, Iterable, List, Optional

import pandas as pd
from Bio import SeqIO

from .schemas import RawSequence


def load_sequences_from_fasta(path: Path) -> Dict[str, str]:
    """Read sequences from a FASTA file and return mapping from ID to sequence."""
    records = {}
    for record in SeqIO.parse(str(path), "fasta"):
        records[record.id] = str(record.seq).upper()
    if not records:
        raise ValueError(f"No sequences found in FASTA file: {path}")
    return records


def load_metadata_from_csv(path: Path) -> pd.DataFrame:
    """Load metadata from CSV/TSV using pandas."""
    delimiter = "\t" if path.suffix.lower() in {".tsv", ".txt"} else ","
    df = pd.read_csv(path, delimiter=delimiter)
    if "id" not in df.columns:
        raise ValueError("Metadata file must contain an 'id' column")
    return df


def merge_metadata_with_sequences(
    sequences: Dict[str, str],
    metadata: Optional[pd.DataFrame] = None,
) -> List[RawSequence]:
    """Merge sequence data with optional metadata dataframe."""
    records: List[RawSequence] = []

    if metadata is None:
        for identifier, sequence in sequences.items():
            records.append(
                RawSequence(
                    id=identifier,
                    name=identifier,
                    lineage="Unknown",
                    location="Unknown",
                    sequence=sequence,
                )
            )
        return records

    metadata = metadata.copy()
    metadata["id"] = metadata["id"].astype(str)

    for identifier, sequence in sequences.items():
        meta_row = metadata.loc[metadata["id"] == identifier]
        if meta_row.empty:
            records.append(
                RawSequence(
                    id=identifier,
                    name=identifier,
                    lineage="Unknown",
                    location="Unknown",
                    sequence=sequence,
                )
            )
            continue

        row = meta_row.iloc[0].to_dict()
        records.append(
            RawSequence(
                id=identifier,
                name=row.get("name") or identifier,
                lineage=row.get("lineage") or row.get("pangolin_lineage") or "Unknown",
                location=row.get("location") or row.get("country") or "Unknown",
                collection_date=row.get("collection_date"),
                sequence=sequence,
            )
        )

    return records


def iter_in_chunks(items: List[RawSequence], chunk_size: int) -> Iterable[List[RawSequence]]:
    """Yield sequences in chunks for batch processing."""
    for i in range(0, len(items), chunk_size):
        yield items[i : i + chunk_size]



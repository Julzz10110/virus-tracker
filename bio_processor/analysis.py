from __future__ import annotations

from dataclasses import dataclass
from typing import Dict, Iterable, List, Optional

from Bio.Align import AlignInfo, MultipleSeqAlignment
from Bio.Phylo.TreeConstruction import DistanceCalculator, DistanceTreeConstructor
from Bio.Seq import Seq
from Bio.SeqRecord import SeqRecord


@dataclass
class AlignmentSummary:
    consensus: str
    length: int
    record_count: int


def _to_alignment_records(sequences: Iterable[Dict[str, str]]) -> List[SeqRecord]:
    records: List[SeqRecord] = []
    for sequence in sequences:
        seq_value = sequence.get("sequence") or ""
        identifier = sequence.get("id") or sequence.get("name") or "unknown"
        records.append(SeqRecord(Seq(seq_value.upper()), id=str(identifier)))
    return records


def build_alignment(
    sequences: List[Dict[str, str]],
) -> MultipleSeqAlignment:
    records = _to_alignment_records(sequences)
    if not records:
        raise ValueError("No sequences provided for alignment")
    return MultipleSeqAlignment(records)


def summarize_alignment(
    alignment: MultipleSeqAlignment,
) -> AlignmentSummary:
    summary = AlignInfo.SummaryInfo(alignment)
    consensus = str(summary.dumb_consensus())
    return AlignmentSummary(
        consensus=consensus,
        length=alignment.get_alignment_length(),
        record_count=len(alignment),
    )


def build_phylogenetic_tree(
    alignment: MultipleSeqAlignment,
    method: str = "neighbor",
) -> Dict:
    calculator = DistanceCalculator("identity")
    distance_matrix = calculator.get_distance(alignment)

    constructor = DistanceTreeConstructor()
    if method == "upgma":
        tree = constructor.upgma(distance_matrix)
    else:
        tree = constructor.nj(distance_matrix)

    return tree_to_json(tree.root)


def tree_to_json(clade) -> Dict:
    node = {
        "name": clade.name or "",
        "branch_length": float(clade.branch_length or 0.0),
    }

    if clade.clades:
        node["children"] = [tree_to_json(child) for child in clade.clades]

    return node


@dataclass
class GrowthMetrics:
    total_count: int
    window_days: int
    doubling_time: Optional[float]
    growth_rate: float
    reproduction_number: Optional[float]


def compute_growth_metrics(
    daily_counts: List[int],
    window_days: int = 7,
    generation_time: float = 4.0,
) -> GrowthMetrics:
    """Примерно оцениваем рост и эффективный R на основе временного ряда."""
    if not daily_counts:
        raise ValueError("daily_counts must not be empty")
    total = sum(daily_counts)

    if len(daily_counts) < window_days * 2:
        return GrowthMetrics(total, window_days, None, 0.0, None)

    recent = sum(daily_counts[-window_days:])
    previous = sum(daily_counts[-2 * window_days : -window_days])
    if previous == 0 or recent == 0:
        return GrowthMetrics(total, window_days, None, 0.0, None)

    growth_rate = recent / previous

    doubling_time = None
    reproduction_number = None
    if growth_rate > 1:
        doubling_time = window_days * (0.693 / (growth_rate - 1 + 1e-9))
        reproduction_number = growth_rate ** (generation_time / window_days)

    return GrowthMetrics(total, window_days, doubling_time, growth_rate, reproduction_number)


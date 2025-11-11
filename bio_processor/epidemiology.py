from __future__ import annotations

from dataclasses import dataclass
from datetime import datetime
from typing import Dict, Iterable, List

import pandas as pd

from .analysis import compute_growth_metrics


@dataclass
class TimeSeriesPoint:
    date: datetime
    count: int


def build_daily_counts(variants: Iterable[Dict]) -> List[TimeSeriesPoint]:
    df = pd.DataFrame(variants)
    if "date" not in df.columns:
        raise ValueError("Variants data must contain 'date'")

    df["date"] = pd.to_datetime(df["date"], errors="coerce")
    df = df.dropna(subset=["date"])
    df["date"] = df["date"].dt.normalize()

    grouped = df.groupby("date").size().sort_index()
    return [TimeSeriesPoint(date=index.to_pydatetime(), count=int(value)) for index, value in grouped.items()]


def aggregate_by_location(variants: Iterable[Dict]) -> pd.DataFrame:
    df = pd.DataFrame(variants)
    if "location" not in df.columns:
        raise ValueError("Variants data must contain 'location'")
    return df.groupby("location").size().reset_index(name="count").sort_values("count", ascending=False)


def compute_growth_summary(variants: Iterable[Dict], window_days: int = 7) -> Dict:
    series = build_daily_counts(variants)
    counts = [point.count for point in series]
    metrics = compute_growth_metrics(counts, window_days=window_days)
    return {
        "total_count": metrics.total_count,
        "window_days": metrics.window_days,
        "doubling_time": metrics.doubling_time,
        "growth_rate": metrics.growth_rate,
        "reproduction_number": metrics.reproduction_number,
        "series": [
            {"date": point.date.isoformat(), "count": point.count} for point in series
        ],
    }


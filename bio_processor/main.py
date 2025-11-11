import asyncio
from pathlib import Path
from typing import Dict, List, Optional

from fastapi import FastAPI, HTTPException, Query
from pydantic import BaseModel

from .analysis import (
    AlignmentSummary,
    build_alignment,
    build_phylogenetic_tree,
    summarize_alignment,
)
from .backend_client import BackendClient
from .config import BackendSettings, IngestionSettings
from .epidemiology import aggregate_by_location, compute_growth_summary
from .pipeline import run_ingestion_pipeline
from .utils import load_reference_sequence

app = FastAPI(title="Bio Processor")

backend_settings = BackendSettings()


class PhylogeneticRequest(BaseModel):
    sequences: List[Dict[str, str]]
    method: str = "neighbor"  # "upgma" или "neighbor"


class MutationAnalysisRequest(BaseModel):
    reference: str
    variants: List[str]


class IngestionRequest(BaseModel):
    sequences_path: str
    metadata_path: Optional[str] = None
    reference_path: Optional[str] = None
    base_url: Optional[str] = None
    dry_run: bool = False
    chunk_size: Optional[int] = None
    update_reference: Optional[bool] = None


def _get_backend_client(base_url: Optional[str] = None) -> BackendClient:
    settings = BackendSettings()
    if base_url:
        settings.base_url = base_url
    return BackendClient(
        base_url=settings.base_url,
        timeout_seconds=settings.timeout_seconds,
    )


@app.post("/api/phylogenetic-tree")
async def build_phylogenetic_tree_endpoint(request: PhylogeneticRequest):
    """Построение филогенетического дерева из набора последовательностей."""
    try:
        alignment = build_alignment(request.sequences)
        tree_data = build_phylogenetic_tree(alignment, method=request.method)
        summary = summarize_alignment(alignment)
        return {
            "tree": tree_data,
            "alignment": summary.__dict__,
            "method": request.method,
        }
    except Exception as exc:
        raise HTTPException(status_code=500, detail=str(exc)) from exc


@app.get("/api/backend/phylogeny")
async def backend_phylogeny(
    method: str = Query("neighbor", enum=["neighbor", "upgma"]),
    lineage: Optional[str] = None,
    location: Optional[str] = None,
):
    """Филогенетическое дерево по вариантам из Rust backend."""
    client = _get_backend_client()
    filters = {}
    if lineage:
        filters["lineage"] = lineage
    if location:
        filters["location"] = location

    try:
        variants = await asyncio.to_thread(client.fetch_variants, filters)
    except Exception as exc:
        raise HTTPException(status_code=502, detail=str(exc)) from exc

    if not variants:
        raise HTTPException(status_code=404, detail="No variants available")

    sequences = [{"id": variant["id"], "sequence": variant["sequence"]} for variant in variants]

    alignment = build_alignment(sequences)
    tree = build_phylogenetic_tree(alignment, method=method)
    summary = summarize_alignment(alignment)
    return {
        "tree": tree,
        "alignment": summary.__dict__,
        "count": len(sequences),
    }


@app.get("/api/backend/alignment")
async def backend_alignment_summary(
    lineage: Optional[str] = None,
    location: Optional[str] = None,
):
    """Сводка по множественному выравниванию вариантов из backend."""
    client = _get_backend_client()
    filters = {}
    if lineage:
        filters["lineage"] = lineage
    if location:
        filters["location"] = location

    try:
        variants = await asyncio.to_thread(client.fetch_variants, filters)
    except Exception as exc:
        raise HTTPException(status_code=502, detail=str(exc)) from exc

    if not variants:
        raise HTTPException(status_code=404, detail="No variants available")

    sequences = [{"id": variant["id"], "sequence": variant["sequence"]} for variant in variants]
    alignment = build_alignment(sequences)
    summary = summarize_alignment(alignment)
    return summary.__dict__


@app.get("/api/backend/epidemiology/growth")
async def backend_growth_summary(
    lineage: Optional[str] = None,
    location: Optional[str] = None,
    window_days: int = 7,
):
    """Временной ряд и метрики роста по данным backend."""
    client = _get_backend_client()
    filters = {}
    if lineage:
        filters["lineage"] = lineage
    if location:
        filters["location"] = location

    try:
        variants = await asyncio.to_thread(client.fetch_variants, filters)
    except Exception as exc:
        raise HTTPException(status_code=502, detail=str(exc)) from exc

    if not variants:
        raise HTTPException(status_code=404, detail="No variants available")

    return compute_growth_summary(variants, window_days=window_days)


@app.get("/api/backend/epidemiology/locations")
async def backend_location_summary(
    lineage: Optional[str] = None,
):
    """Агрегация по локациям для оценки географического распространения."""
    client = _get_backend_client()
    filters = {}
    if lineage:
        filters["lineage"] = lineage

    try:
        variants = await asyncio.to_thread(client.fetch_variants, filters)
    except Exception as exc:
        raise HTTPException(status_code=502, detail=str(exc)) from exc

    if not variants:
        raise HTTPException(status_code=404, detail="No variants available")

    df = aggregate_by_location(variants)
    return df.to_dict(orient="records")


@app.post("/api/advanced-mutations")
async def analyze_advanced_mutations(request: MutationAnalysisRequest):
    """Расширенный анализ мутаций с использованием эвристик."""
    mutations = []

    for i, (ref_base, var_base) in enumerate(zip(request.reference, request.variants[0])):
        if ref_base != var_base:
            start = max(0, i - 5)
            end = min(len(request.reference), i + 6)
            context = request.reference[start:end]

            mutation = {
                "position": i,
                "reference": ref_base,
                "mutation": var_base,
                "context": context,
                "type": classify_mutation_type(ref_base, var_base),
                "impact_score": calculate_impact_score(i, ref_base, var_base, context),
            }
            mutations.append(mutation)

    return {
        "total_mutations": len(mutations),
        "mutations": mutations,
        "mutation_rate": len(mutations) / len(request.reference),
    }


@app.post("/api/ingest-local")
async def ingest_local_dataset(request: IngestionRequest):
    """Загрузить локальные последовательности в backend через пайплайн."""
    try:
        ingestion_settings = IngestionSettings(
            sequences_path=Path(request.sequences_path),
            metadata_path=Path(request.metadata_path) if request.metadata_path else None,
            dry_run=request.dry_run,
            chunk_size=request.chunk_size or 50,
        )

        backend_settings = BackendSettings()
        if request.base_url:
            backend_settings.base_url = request.base_url
        if request.update_reference is not None:
            backend_settings.update_reference = request.update_reference

        reference_sequence = load_reference_sequence(
            Path(request.reference_path) if request.reference_path else None
        )

        report = await asyncio.to_thread(
            run_ingestion_pipeline,
            ingestion_settings,
            backend_settings,
            reference_sequence,
        )
        return report.dict()
    except Exception as exc:
        raise HTTPException(status_code=500, detail=str(exc)) from exc


def classify_mutation_type(ref: str, mut: str) -> str:
    """Классификация типа мутации."""
    transitions = [("A", "G"), ("G", "A"), ("C", "T"), ("T", "C")]
    return "transition" if (ref, mut) in transitions else "transversion"


def calculate_impact_score(position: int, ref: str, mut: str, context: str) -> float:
    """Упрощенная оценка воздействия мутации."""
    score = 0.0
    if position % 3 == 0:
        score += 0.3
    if "ATG" in context or "TGG" in context:
        score += 0.2
    return min(score, 1.0)


@app.get("/health")
async def health():
    return {"status": "healthy"}


if __name__ == "__main__":
    import uvicorn

    uvicorn.run(app, host="0.0.0.0", port=8000)
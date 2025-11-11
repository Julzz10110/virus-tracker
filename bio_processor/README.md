# Bio Processor – Virus Evolution Tracker

FastAPI service providing bioinformatics and epidemiological computations for the Virus Evolution Tracker platform.

---

## Capabilities

- Fetches variants from the Rust backend and derives:
  - Multiple sequence alignments
  - Phylogenetic trees (UPGMA/Neighbor-Joining)
  - Alignment summaries (consensus, length, record count)
  - Epidemiological metrics (growth rate, Rₑ, doubling time) aggregated over configurable windows
  - Location-based aggregations
- Hosts programmatic ingestion helpers for batch-loading local datasets into the backend.
- Exposes a REST API consumed by the Axum backend and directly by clients.

---

## Endpoints

| Route | Method | Description |
| ----- | ------ | ----------- |
| `/api/backend/epidemiology/growth` | GET | Compute growth summaries using variants pulled from the backend |
| `/api/backend/epidemiology/locations` | GET | Aggregate variants by location |
| `/api/backend/phylogeny` | GET | Build phylogenetic tree & alignment summary |
| `/api/backend/alignment` | GET | Return alignment summary only |
| `/api/phylogenetic-tree` | POST | Build tree/alignment from a supplied sequence list |
| `/api/advanced-mutations` | POST | Experimental mutation impact analysis |
| `/api/ingest-local` | POST | Run ingestion pipeline on local sequence files (CSV/FASTA) |
| `/health` | GET | Health probe |

Most `/api/backend/*` routes internally fetch variant data from the Rust backend using the configured base URL and optional filters (`lineage`, `location`). If the backend has no matching variants, the service responds with `404 { "detail": "No variants available" }`.

---

## Configuration

Environment variables (defaults in `config.py`):

| Variable | Description | Default |
| -------- | ----------- | ------- |
| `BACKEND_BASE_URL` | Base REST endpoint of the Rust backend | `http://localhost:8080/api` |
| `BACKEND_TIMEOUT_SECONDS` | Timeout for backend requests | `10` |
| `UPDATE_REFERENCE` | Whether ingestion pipeline should update the backend reference sequence | `False` |
| `INGESTION_CHUNK_SIZE` | Batch size when pushing variants during ingestion | `50` |

Other ingestion settings (paths, dry-run, etc.) are supplied in the request body of `/api/ingest-local`.

---

## Local Development

### Prerequisites
- Python 3.11+
- Virtual environment (recommended)

### Setup
```bash
cd bio_processor
python -m venv .venv
. .venv/bin/activate  # .venv\Scripts\Activate.ps1 on Windows
pip install -r requirements.txt
uvicorn bio_processor.main:app --reload --host 0.0.0.0 --port 8000
```

The service will be available at <http://localhost:8000>.

### Integration with backend
Ensure the Rust backend is running and accessible at `BACKEND_BASE_URL`. For Docker Compose the hostname `http://backend:8080/api` is injected automatically.

When the backend has no variants that match the requested filters, growth/phylogeny endpoints return `404`. The Axum API handles these responses gracefully and supplies empty payloads to the UI.

---

## Ingestion Pipeline

`/api/ingest-local` accepts paths to local FASTA/CSV files and optional metadata. The pipeline:
1. Validates file existence.
2. Optionally loads a reference sequence (if provided).
3. Processes sequences in batches (`chunk_size`).
4. Uses `backend_client.py` to push variants into the Rust API (respecting `update_reference`).

Example payload:
```json
{
  "sequences_path": "data/sequences.fasta",
  "metadata_path": "data/metadata.csv",
  "reference_path": "data/reference.fasta",
  "chunk_size": 100,
  "dry_run": false,
  "update_reference": true
}
```

---

## Testing & Linting

Add Python tests under `tests/` (not yet scaffolded) and run with `pytest`. Style-formatters such as `black`/`isort` are recommended but not enforced in this repository.

---

## Deployment Notes

- Dockerfile installs dependencies from `requirements.txt`, copies the service into `/app/bio_processor`, sets `PYTHONPATH=/app`, and starts `uvicorn bio_processor.main:app`.
- For production, configure timeouts, base URL, and TLS termination externally (e.g. via reverse proxy or ingress controller).
- Scale horizontally as needed; the service is stateless and fetches data from the backend on demand.

---

## Useful Commands

```bash
# Run in reload mode
uvicorn bio_processor.main:app --reload --host 0.0.0.0 --port 8000

# Call growth endpoint with filters
http :8000/api/backend/epidemiology/growth lineage==B.1 location==Europe

# Phylogeny using supplied sequences
http POST :8000/api/phylogenetic-tree method=neighbor sequences:='[{"id":"sample","sequence":"ACGT"}]'
```

Refer back to the root project README and OpenAPI spec for end-to-end context and API schemas.

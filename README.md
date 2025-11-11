# Virus Evolution Tracker

A full-stack platform for monitoring viral evolution, epidemiological trends, and phylogenetic structure. The system combines a Rust (Axum) API, a Python bioinformatics service, and a Vue 3 dashboard to ingest sequences, compute analytics, and expose actionable insights for researchers and public-health teams.

---

## Key Features

- **Variant catalogue** – upload and manage genomic sequences with lineage, location and metadata.
- **Mutation analytics** – compute per-position mutation frequencies and entropy against a configurable reference genome.
- **Phylogeny & alignment** – run multiple sequence alignment and tree reconstruction through the Python bio processor.
- **Epidemiological metrics** – calculate reproduction number (Rₑ), growth rates and doubling time; store snapshots for trend analysis.
- **RBAC & user management** – JWT-based authentication, role-restricted actions, and an admin UI for provisioning users.
- **Observability** – Prometheus metrics, structured tracing logs, configurable alert threshold for Rₑ growth.

---

## Architecture Overview

| Layer | Technology | Responsibilities |
| ----- | ---------- | ---------------- |
| Backend (`backend/`) | Rust, Axum, sqlx, JWT, Prometheus | REST API, persistence (SQLite), RBAC, background ingestion of epidemiological metrics, `/metrics` endpoint |
| Bio Processor (`bio_processor/`) | Python, FastAPI, BioPython, pandas | Alignment & phylogeny generation, epidemiological summaries, ingestion pipeline, backend client |
| Frontend (`frontend/`) | Vue 3, Pinia, Axios, Plotly | Dashboard, variant explorer, analytics visualisation, login & admin UX |
| Deploy | Docker Compose, Helm chart | Local orchestration, Kubernetes manifests |
| Docs | `docs/openapi.yaml` | OpenAPI 3.0 specification for the REST API |

Each component includes its own README with deeper details:

- [`backend/README.md`](backend/README.md)
- [`bio_processor/README.md`](bio_processor/README.md)
- [`frontend/README.md`](frontend/README.md)

---

## Quick Start (Docker Compose)

```bash
# From the repo root
docker compose up --build
```

Services will be available at:

- Backend API – <http://localhost:8080>
- Bio Processor – <http://localhost:8000>
- Frontend UI – <http://localhost:8081>

The Compose file seeds a demo API key (`demo-ingest-key`). **Generate your own JWT secret and API key before deploying anywhere outside development.**

---

## Manual Development Setup

### Backend (Rust)
```bash
cd backend
cargo run
```
Set environment variables (`JWT_SECRET`, `DATABASE_URL`, etc.) before launching.

### Bio Processor (Python)
```bash
cd bio_processor
python -m venv .venv
. .venv/bin/activate  # .venv\Scripts\Activate.ps1 on Windows
pip install -r requirements.txt
uvicorn bio_processor.main:app --reload --host 0.0.0.0 --port 8000
```

### Frontend (Vue 3)
```bash
cd frontend
npm install
npm run dev -- --host
```
Dev server listens on <http://localhost:5173> (configurable) and uses `VITE_API_BASE_URL` for API calls.

---

## Authentication & Roles

| Role | Capabilities |
| ---- | ------------ |
| `admin` | Manage users, upload variants, update reference sequence, view analytics |
| `uploader` | Upload variants, view analytics |
| `analyst` | Read-only access to analytics and metrics |
| `viewer` | Read-only access to public dashboards |

Compose injects `API_KEY=demo-ingest-key`. Any request with header `x-api-key: demo-ingest-key` is treated as an administrator – use this to bootstrap accounts:

```bash
curl -X POST http://localhost:8080/api/admin/users \
  -H "Content-Type: application/json" \
  -H "x-api-key: demo-ingest-key" \
  -d '{"email":"admin@example.com","password":"ChangeMe123","roles":["admin","uploader"]}'
```

Then authenticate via UI or API:
```bash
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@example.com","password":"ChangeMe123"}'
```
The JSON response contains a JWT (`token`) and assigned roles – store it in local storage for the frontend or include it in the `Authorization: Bearer ...` header for API calls.

---

## API Documentation

The full REST contract lives in [`docs/openapi.yaml`](docs/openapi.yaml). Render with ReDoc:
```bash
npx redoc-cli serve docs/openapi.yaml
```
Primary endpoints:

- `/api/variants`, `/api/variants/stats` – variant catalogue and aggregated statistics
- `/api/analyze`, `/api/analysis/frequency` – mutation detection and frequency analysis
- `/api/analysis/phylogeny` – phylogenetic tree and alignment summary (proxied from bio processor)
- `/api/metrics/growth`, `/api/metrics/growth/live` – historical snapshots and live epidemiological metrics
- `/api/auth/login`, `/api/admin/users` – authentication & user management
- `/metrics` – Prometheus exposition endpoint for scraping

Background tasks poll the bio processor hourly to persist growth metrics; adjust interval and thresholds via environment variables.

---

## Observability

- **Prometheus metrics:** <http://localhost:8080/metrics>
- **Structured tracing:** controlled via `RUST_LOG` (e.g., `info`, `debug`).
- **Alerting:** when Rₑ exceeds `GROWTH_ALERT_THRESHOLD`, logs emit a warning on the `alerts` target.

---

## Testing

| Component | Command |
| --------- | ------- |
| Backend | `cargo test` |
| Frontend | `npm run build` (and optional `npm run lint` if configured) |
| Bio Processor | `pytest` (add tests as the pipeline evolves) |
| Compose stack smoke test | `docker compose up --build` followed by manual UI/API verification |

---

## Environment Variables Summary

| Variable | Service | Description | Default |
| -------- | ------- | ----------- | ------- |
| `DATABASE_URL` | backend | SQLx connection string | `sqlite://data/virus_tracker.db` |
| `BIO_PROCESSOR_URL` | backend | Base URL of Python analytics service | `http://localhost:8000` |
| `JWT_SECRET` | backend | JWT signing secret | _required_ |
| `API_KEY` | backend | Optional static API key for privileged requests | unset |
| `TOKEN_TTL_HOURS` | backend | JWT lifetime in hours | `24` |
| `GROWTH_ALERT_THRESHOLD` | backend | Rₑ warning threshold | `1.2` |
| `BACKEND_BASE_URL` | bio processor | Backend base URL (used by internal client) | `http://localhost:8080/api` |
| `BACKEND_TIMEOUT_SECONDS` | bio processor | Request timeout | `10` |
| `VITE_API_BASE_URL` | frontend | API base URL for the SPA | `http://localhost:8080/api` |
| `VITE_API_KEY` | frontend | Optional API key to include with requests | unset |

---

## Repository Layout

```
.
├── backend/                 # Rust Axum service (API, storage, tasks)
├── bio_processor/           # FastAPI analytics service
├── frontend/                # Vue 3 single-page application
├── docs/openapi.yaml        # OpenAPI 3.0 definition
├── deploy/                  # Helm chart and templates
├── docker-compose.yml       # Local stack composition
└── README.md                # You are here
```

---

## Next Steps & Contribution Guidelines

1. Fork and branch (`git checkout -b feature/xyz`).
2. Keep code formatted (`cargo fmt`, `npm run lint`, `black`/`isort` for Python as needed).
3. Update documentation and OpenAPI whenever API contracts change.
4. Run tests (component + integration) before opening a PR.
5. Coordinate larger roadmap items with the maintainers – see the project summary and future plan at the end of this document.

---

For detailed component instructions, see the READMEs within each subdirectory. A project summary and future roadmap are provided at the end of this response.

# Backend – Virus Evolution Tracker

Rust/Axum service providing the core REST API, persistence, authentication and background jobs for the Virus Evolution Tracker platform.

---

## Features

- **REST API** for variant ingestion, analytics and administration (documented in `docs/openapi.yaml`).
- **SQLite persistence** via `sqlx` with typed queries and migrations.
- **Authentication & RBAC** using JWT, API keys, Argon2 password hashing, and role enforcement middleware.
- **Background tasks** that poll the bio processor for epidemiological metrics and persist time-series snapshots.
- **Prometheus metrics & tracing** exposed via `/metrics` and `tracing` instrumentation.

---

## Project Structure

```
src/
├── main.rs              # Application bootstrap
├── lib.rs               # Crate exports
├── state.rs             # Shared application state (storage, secrets, metrics handle)
├── storage.rs           # SQLx data access layer and migrations helper
├── models.rs            # Domain DTOs (variants, metrics, payloads)
├── routes/              # Route handlers (variants, analysis, auth, metrics, health)
├── services/            # Background tasks, analysis helpers
├── auth.rs              # JWT, API key extraction, role guards
├── errors.rs            # Error handling utilities
└── tests/               # Integration smoke tests
```

Database migrations live in `migrations/*.sql` and are executed automatically at startup (`Storage::migrate`).

---

## Environment Variables

| Variable | Description | Default |
| -------- | ----------- | ------- |
| `DATABASE_URL` | SQLx connection string (use `sqlite://...` or other drivers) | `sqlite://data/virus_tracker.db` |
| `JWT_SECRET` | Symmetric signing secret for JWTs | _required_ |
| `API_KEY` | Optional static API key granting admin privileges | unset |
| `TOKEN_TTL_HOURS` | Lifetime of issued JWT tokens | `24` |
| `BIO_PROCESSOR_URL` | Base URL of the Python analytics service | `http://localhost:8000` |
| `GROWTH_ALERT_THRESHOLD` | Rₑ threshold for alert logging | `1.2` |
| `RUST_LOG` | Log level (e.g. `info`, `debug`, `trace`) | `info` |

Supply variables via `.env`, shell exports, Docker Compose, or Kubernetes manifests.

---

## Running Locally

### Prerequisites

- Rust toolchain (1.83 or newer recommended)
- SQLite (embedded via `sqlx`; no external server required)

### Start the server

```bash
# From repo root
cd backend
cargo run
```

The server listens on `0.0.0.0:8080` by default. At startup it:
1. Creates the SQLite database path if necessary.
2. Runs pending migrations.
3. Installs the Prometheus recorder.
4. Spawns background metric collection tasks.

A healthy instance responds to:
- <http://localhost:8080/api/health>
- <http://localhost:8080/metrics>

### Authentication bootstrap

Use the API key to create the first admin account:
```bash
curl -X POST http://localhost:8080/api/admin/users \
  -H "Content-Type: application/json" \
  -H "x-api-key: demo-ingest-key" \
  -d '{"email":"admin@example.com","password":"ChangeMe123","roles":["admin","uploader"]}'
```
Login to obtain a JWT:
```bash
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"admin@example.com","password":"ChangeMe123"}'
```

### Background jobs

`services::tasks::spawn_background_jobs` launches an async loop (default hourly) that hits
`{BIO_PROCESSOR_URL}/api/backend/epidemiology/growth`, stores the response in SQLite, increments Prometheus counters, and emits an alert if `reproduction_number` surpasses `GROWTH_ALERT_THRESHOLD`.
Adjust the interval or behaviour by editing `services/tasks.rs`.

---

## Metrics & Observability

- **Prometheus endpoint:** `GET /metrics` (content type `text/plain; version=0.0.4`).
- **Tracing:** configure `RUST_LOG` (e.g. `RUST_LOG=virus_tracker_axum=debug,tower_http=info`).
- **Alert logging:** warnings are logged with `target="alerts"` when Rₑ exceeds the configured threshold.

Example metrics emitted:
```
growth_metrics_collected_total 5
growth_total_variants 12
reproduction_number_latest 1.08
```

---

## Database & Migrations

SQLite is the default driver; use `sqlx migrate add <name>` to create new migrations. Migrations run at startup via `Storage::migrate()`. For local inspection:

```bash
sqlite3 data/virus_tracker.db ".tables"
```

Tables include `variants`, `mutations`, `users`, `roles`, `user_roles`, `epidemiology_metrics`, etc.

---

## Testing

```bash
cargo test
```
`tests/api_smoke.rs` boots an in-memory SQLite database and exercises the public API (health, upload, stats, mutation frequency). Extend with additional scenarios as needed.

Run `cargo fmt` and `cargo clippy` before committing changes.

---

## Deployment Notes

- Dockerfile builds a static release binary and copies it into a slim Debian image.
- Helm manifests under `deploy/helm/virus-tracker/templates` provision ConfigMaps, Secrets, Deployments and Services for Kubernetes.
- Ensure `JWT_SECRET` and `API_KEY` are injected via secrets, and configure persistent storage for SQLite (or switch to PostgreSQL/MySQL by enabling the relevant `sqlx` features).

---

## Useful Endpoints

| Endpoint | Method | Description | Auth |
| -------- | ------ | ----------- | ---- |
| `/api/health` | GET | Liveness probe | None |
| `/api/auth/login` | POST | JWT login | None |
| `/api/admin/users` | GET/POST | List & create users | Admin JWT or API key |
| `/api/variants` | GET | List variants (filter by lineage/location) | Optional |
| `/api/upload` | POST | Upload new variant | Admin/uploader role or API key |
| `/api/metrics/growth` | GET | Historical growth metrics | Optional |
| `/api/metrics/growth/live` | GET | Live snapshot proxied from bio processor | Optional |
| `/metrics` | GET | Prometheus scrape endpoint | None |

Refer to the root `README.md` and `docs/openapi.yaml` for the full API contract and request/response schemas.

# Frontend – Virus Evolution Tracker

Vue 3 single-page application delivering dashboards, analytics, and admin UX for the Virus Evolution Tracker platform.

---

## Features

- **Dashboard** – global statistics, mutation frequency chart, epidemiological trends, reference-sequence management, variant upload form.
- **Variants** – tabular catalogue of uploaded variants with refreshed views.
- **Analysis** – filtered mutation frequency/entropy table for deeper inspection.
- **Phylogeny** – tree and alignment summaries fetched from the bio processor.
- **Authentication & Admin** – login flow, JWT storage, user management screen (create/list users by role).
- **State management** via Pinia stores (auth, variants, analytics) with Axios-based API client.

---

## Technology Stack

- Vue 3 + `<script setup>` single-file components
- Pinia for global stores
- Vue Router for navigation
- Axios for REST interaction (`src/services/api.js`)
- Plotly.js for charts (mutation frequency, epidemiological trends)
- Nginx (production build via multi-stage Dockerfile)

---

## Getting Started

### Development
```bash
cd frontend
npm install
npm run dev -- --host
```
The dev server runs at <http://localhost:5173> by default. API requests proxy to `VITE_API_BASE_URL` (set in `.env` or via npm scripts).

### Production build
```bash
npm run build
npm run preview   # optional, serves dist on http://localhost:4173
```
Dockerfile builds the assets and serves them through Nginx for Compose/Kubernetes deployments.

---

## Configuration

Environment variables (prefix `VITE_` – must be set at build time):

| Variable | Description | Default |
| -------- | ----------- | ------- |
| `VITE_API_BASE_URL` | Base URL of the backend API (`/api` prefix included) | `http://localhost:8080/api` |
| `VITE_API_KEY` | Optional API key forwarded as `x-api-key` header (useful for dev shortcuts) | unset |

Create a `.env` file or pass build args in Docker Compose/Helm.

---

## Authentication Flow

1. User submits credentials to `/api/auth/login` via `authStore`.
2. JWT token and roles are stored in `localStorage` (`auth_token`, `auth_roles`).
3. Axios interceptor appends `Authorization: Bearer <token>` (and `x-api-key` if provided).
4. Route guards (Vue Router `beforeEach`) enforce role requirements for admin routes.
5. Logging out clears local storage and redirects to `/login`.

Roles are surfaced in the UI to enable/disable upload, reference editing, and admin pages.

---

## Project Structure

```
src/
├── App.vue                 # Shell layout and navigation
├── main.js                 # App bootstrap, plugins, global polyfills
├── router/                 # Route definitions & guards
├── store/                  # Pinia stores (auth, variants, analytics)
├── services/api.js         # Axios instance with interceptors
├── components/             # Reusable components (e.g., VariantDashboard)
├── views/                  # Routed pages (Dashboard, Variants, Analysis, Phylogeny, Login, Admin)
└── styles/                 # Global CSS
```

Each view consumes Pinia stores which in turn orchestrate API calls.

---

## Testing & Linting

Currently the project relies on `npm run build` to ensure type/compile correctness. For additional quality gates consider adding:
- `npm run lint` (ESLint/Prettier)
- Component tests with Vitest/Cypress

---

## Integration Tips

- Ensure the backend API (`VITE_API_BASE_URL`) is reachable from the browser: when serving via Docker/Nginx, use `http://localhost:8080/api` (not the internal service name like `http://backend:8080/api`).
- If bio-processor has no variants yet, analytics views will show `N/A` or informative placeholders instead of failing.
- Admin-only capabilities (reference updates, user management, variant upload) are guarded in the UI; use an admin account or API key to test them.

---

## Useful Commands

```bash
npm run dev        # start dev server with HMR
npm run build      # generate production bundle
npm run preview    # serve built assets locally
```

For complete system instructions, refer to the root [`README.md`](../README.md) and the backend/bio-processor documentation.

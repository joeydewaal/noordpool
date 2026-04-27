# Noordpool

Football team PWA for "De Noordpool". Players can view matches, results, stats, and receive live goal notifications.

## Project structure

- `backend/` — Rust (Axum) API server
- `frontend/` — SvelteKit (Svelte 5) PWA with Tailwind CSS and Skeleton UI

## Language convention

- **UI text**: Dutch (nl-NL) — all labels, messages, buttons, headings
- **Code**: English — variables, functions, comments, models, API fields
- **Date/time**: nl-NL locale formatting

## Running locally

Backend (port 3000):

```sh
cd backend && cargo run
```

Reads `backend/.env` for `DATABASE_URL`, `JWT_SECRET`, VAPID keys, etc. Defaults to `sqlite::memory:` if `DATABASE_URL` is unset. Schema auto-migration runs on every startup. Setting `NOORDPOOL_SEED=1` also seeds an admin user (`admin@noordpool.be` / `Admin123`) and demo data — leave it unset in production.

Frontend (port 5173):

```sh
cd frontend && npm run dev
```

Proxies `/api` requests to `http://localhost:3000`.

## Backend patterns

### Module layout

Each resource is a module under `src/` with:
- `mod.rs` — defines `pub fn router() -> Router<AppState>` with route definitions
- `handlers.rs` — handler functions extracted by Axum
- Models live in `src/models/` using the toasty ORM

Routes are nested in `src/routes.rs` via `Router::nest("/api/<resource>", <module>::router())`.

### Auth and roles

Uses `axum-security` for JWT and RBAC. Roles: `Admin`, `Moderator`, `Player`.

```rust
#[requires(Role::Admin)]                          // single role
#[requires_any(Role::Admin, Role::Moderator)]     // any of these roles
```

### Serialization

All request/response structs use camelCase JSON:

```rust
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateGameRequest { ... }
```

### Error handling

Handlers return `Result<impl IntoResponse, AppError>` where `AppError` is in `src/error.rs`.

### Code style

No banner comments. Do not use `// ----...----` or similar horizontal-rule separators to divide sections in source files.

## Frontend patterns

- Data fetching: TanStack Query (`createQuery`) with API functions in `src/lib/api/`
- Auth state: `src/lib/state/auth.svelte.ts` — exposes `auth.isAdmin`, `auth.isModerator`, `auth.user`, etc.
- Components: Skeleton UI (`@skeletonlabs/skeleton-svelte`)
- Tests colocated as `*.test.ts` next to the component/page they test

## Deployment

Deployed to **Fly.io** as a single persistent Axum binary.

- Live match updates use a per-match **WebSocket** at `/api/games/:id/ws` — viewers receive `Snapshot` / `ScoreUpdate` / `EventAdded` / `EventDeleted` / `StatusChange` frames.
- Mutations still go through the existing authenticated REST endpoints (e.g. `POST /api/games/:id/live/score`); the mutation handler publishes to the in-memory hub which fans out to connected sockets.
- Push notifications use Web Push for OS-level alerts when the app is closed/backgrounded — WebSockets cover in-app live state, Web Push covers out-of-app delivery. The push backend uses `web-push` for VAPID + payload encryption (the `ece` crate pulls openssl — unavoidable for AES-GCM/HKDF) but ships the request via `reqwest` + rustls; no `isahc` / `curl-sys` / `native-tls` in the build graph.

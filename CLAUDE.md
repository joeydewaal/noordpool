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

Reads `backend/.env` for `DATABASE_URL`, `JWT_SECRET`, VAPID keys, etc. Defaults to `sqlite::memory:` if `DATABASE_URL` is unset. In dev mode, auto-migrates the schema and seeds an admin user (`admin@noordpool.be` / `Admin123`).

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

## Frontend patterns

- Data fetching: TanStack Query (`createQuery`) with API functions in `src/lib/api/`
- Auth state: `src/lib/state/auth.svelte.ts` — exposes `auth.isAdmin`, `auth.isModerator`, `auth.user`, etc.
- Components: Skeleton UI (`@skeletonlabs/skeleton-svelte`)
- Tests colocated as `*.test.ts` next to the component/page they test

## Deployment

Deployed to **AWS Lambda** with the `prod` feature flag:
- `cargo build --release --features prod` — enables `lambda_http` runtime, disables auto-migration
- **No SSE/WebSockets** — Lambda does not support long-lived connections
- Live match updates use HTTP polling with `ETag` / `304 Not Modified`
- Push notifications use Web Push (fire-and-forget, works from Lambda)

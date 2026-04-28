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

Split deploy:

- **Backend** runs on **Fly.io** as a single persistent Axum binary (`Dockerfile` at repo root, `fly.toml`). Listens on `:3000`, exposes `/api/*`, `/avatars/*`, and `/health`. Live match updates use a per-match **WebSocket** at `/api/games/:id/ws` — viewers receive `Snapshot` / `ScoreUpdate` / `EventAdded` / `EventDeleted` / `StatusChange` frames. Mutations still go through the existing authenticated REST endpoints; the mutation handler publishes to the in-memory hub which fans out to connected sockets. Deploy with `fly deploy`.
- **Frontend** is a static SvelteKit build (`adapter-static`, SPA fallback to `index.html`) deployed to **Cloudflare Pages**. No git integration / no auto-deploy — push to deploy via `cd frontend && npm run deploy` (wraps `wrangler pages deploy build --project-name noordpool`). Set `VITE_API_BASE_URL` in the CF Pages project env to the backend origin.
- **Cross-origin**: backend serves `https://api.noordpool.joeydewaal.com`, frontend serves `https://noordpool.joeydewaal.com`. CORS origins come from the `ALLOWED_ORIGINS` env (comma-separated). JWT lives in `localStorage` and goes out as `Authorization: Bearer …` — no cross-site cookies. The OIDC builder enables `use_dev_cookies(true)` only when the `prod` cargo feature is off.
- **Avatar URLs** are absolute (`PUBLIC_API_URL/avatars/<id>.webp`) when `PUBLIC_API_URL` is set, so the cross-origin frontend can render them.
- **Push notifications** use Web Push (RFC 8030) for OS-level alerts when the app is closed/backgrounded — WebSockets cover in-app live state, Web Push covers out-of-app delivery. The push backend uses `web-push` for VAPID + payload encryption (the `ece` crate pulls openssl — unavoidable for AES-GCM/HKDF) but ships the request via `reqwest` + rustls; no `isahc` / `curl-sys` / `native-tls` in the build graph.

CI builds the Docker image (`docker-build` job) on every PR so a broken Dockerfile fails the check before deploy.

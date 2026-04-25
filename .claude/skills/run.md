---
description: Run the backend and/or frontend dev servers locally
---

## Backend

```sh
cd backend && cargo run
```

- Listens on port 3000 (configurable via `PORT` env var)
- Reads `backend/.env` for config (`DATABASE_URL`, `JWT_SECRET`, VAPID keys, Google OAuth)
- Defaults to `sqlite::memory:` if no `DATABASE_URL` — good for quick local dev
- Auto-migrates schema and seeds admin user in dev mode (non-`prod` feature)

## Frontend

```sh
cd frontend && npm run dev
```

- Listens on port 5173
- Proxies `/api` to `http://localhost:3000` (configured in `vite.config.ts`)
- Run both backend and frontend for full local development

## Production build

Backend:
```sh
cd backend && cargo build --release --features prod
```

Frontend:
```sh
cd frontend && npm run build
```

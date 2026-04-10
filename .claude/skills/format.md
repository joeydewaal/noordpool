---
description: Check and fix formatting and linting for backend and frontend
---

## Backend

Check formatting:
```sh
cd backend && cargo fmt --check
```

Fix formatting:
```sh
cd backend && cargo fmt
```

Lint with clippy (warnings are errors in CI):
```sh
cd backend && cargo clippy -- -D warnings
```

## Frontend

Check formatting:
```sh
cd frontend && npx prettier --check .
```

Fix formatting:
```sh
cd frontend && npx prettier --write .
```

Type check (svelte-check):
```sh
cd frontend && npm run check
```

## Before committing

Run all checks:
```sh
cd backend && cargo fmt --check && cargo clippy -- -D warnings
cd frontend && npx prettier --check . && npm run check
```

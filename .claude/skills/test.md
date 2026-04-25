---
description: Run and write tests for backend and frontend
---

## Running tests

Backend (all tests):
```sh
cd backend && cargo test
```

Frontend (all tests):
```sh
cd frontend && npm run test
```

Run a specific backend test file:
```sh
cd backend && cargo test --test games
```

Run a specific frontend test file:
```sh
cd frontend && npx vitest run src/routes/games/games.test.ts
```

## Backend test patterns

Tests live in `backend/tests/` with a shared harness in `tests/common/mod.rs`.

Every test file starts with:
```rust
mod common;
use crate::common::TestApp;
```

### TestApp harness

```rust
let mut app = TestApp::new().await;
```

Creates an isolated SQLite in-memory database per test.

### Getting auth tokens

```rust
let admin = app.admin_token().await;
let moderator = app.moderator_token().await;
let player = app.player_token().await;
```

### Making requests

```rust
// GET
let res = app.get("/api/players").token(&admin).send().await;

// POST with JSON body
let res = app.post("/api/players").token(&admin).json(json!({
    "firstName": "Test",
    "lastName": "Player",
    "shirtNumber": 10,
    "position": "Spits"
})).await;

// Reading responses
assert_eq!(res.status(), StatusCode::OK);
let body = res.json_value().await;
let typed: Vec<Player> = res.json().await;
```

### Snapshot testing

Uses `insta` for JSON snapshot assertions:
```rust
use insta::{Settings, assert_json_snapshot};

let mut settings = Settings::clone_current();
settings.add_redaction(".id", insta::dynamic_redaction(|val, _| {
    val.as_str().map(|_| "[uuid]".into()).unwrap_or(val.clone())
}));
settings.bind(|| {
    assert_json_snapshot!(body);
});
```

## Frontend test patterns

Tests are colocated as `*.test.ts` next to the component they test. Use vitest + @testing-library/svelte + jsdom.

### Typical test structure

```typescript
import { render, screen } from "@testing-library/svelte";
import { vi, describe, it, expect, beforeEach } from "vitest";
import Page from "./+page.svelte";

// Mock SvelteKit modules and API functions
vi.mock("@tanstack/svelte-query", () => ({
  createQuery: vi.fn(),
}));

vi.mock("$lib/api/games", () => ({
  getUpcomingGames: vi.fn(),
  getRecentResults: vi.fn(),
}));

vi.mock("$lib/state/auth.svelte", () => ({
  auth: { isAdmin: false, isModerator: false },
}));

// Import mocked modules after vi.mock()
import { createQuery } from "@tanstack/svelte-query";

beforeEach(() => {
  vi.clearAllMocks();
});

it("shows loading state", () => {
  vi.mocked(createQuery).mockReturnValue({
    isPending: true, isError: false, data: undefined
  } as ReturnType<typeof createQuery>);

  render(Page);
  expect(screen.getByText("Laden...")).toBeInTheDocument();
});
```

### Key conventions

- Always mock `$lib/state/auth.svelte` to control role visibility
- Mock `$app/state` (page), `$app/navigation` (goto) via the test mocks in `src/test/mocks/app/`
- Mock API functions from `$lib/api/*`, not axios directly
- Mock `createQuery` return values to control loading/success/error states

default:
    @just --list

# Format backend (Rust) and frontend (Prettier)
fmt:
    cd backend && cargo fmt
    cd frontend && npx prettier --write .

# Run all checks: fmt, clippy, typecheck, tests
check:
    cd backend && cargo fmt --check
    cd backend && cargo clippy -- -D warnings
    cd backend && cargo test
    cd frontend && npx prettier --check .
    cd frontend && npx svelte-check
    cd frontend && npx vitest run

# Run backend tests
test-backend:
    cd backend && cargo test

# Run frontend tests
test-frontend:
    cd frontend && npx vitest run

# Run all tests
test: test-backend test-frontend

# Start backend dev server
dev-backend:
    cd backend && cargo run

# Start frontend dev server
dev-frontend:
    cd frontend && npm run dev

# Noordpool - Football Team App Roadmap

## Context

A PWA for football teams where players can view upcoming matches, match results (with goal scorers, assists, cards, timestamps), and personal stats. Matches track both home and away teams with full event tracking for all players. Guest users can view everything read-only. Authenticated users have roles: **Admin** (full control including user/role management), **Moderator** (manage matches & player stats), **Player** (view only). Users can have multiple roles.

## Tech Stack

- **Frontend:** SvelteKit (PWA)
- **Backend:** Rust with Axum, axum-security (local path dep), toasty (local path dep)
- **Database:** PostgreSQL
- **Auth:** Guest access (read-only), email+password, Google OAuth

## Language Convention

- **UI text:** Dutch (nl-NL) — all user-facing labels, messages, buttons, headings
- **Code:** English — variable names, function names, comments, database models, API fields
- **Date/time formatting:** nl-NL locale

---

## Phase 1: Project Scaffolding & Auth -- DONE

### Backend
- [x] Initialize Rust workspace with Axum
- [x] Configure path dependencies for `axum-security` and `toasty`
- [x] Set up PostgreSQL connection (via toasty)
- [x] Database schema: `users` table (id, email, password_hash, first_name, last_name, player_id, avatar_url, is_admin, is_moderator, created_at)
- [x] Role assignment via boolean flags on User model (`is_admin`, `is_moderator`) + computed `get_roles()`
- [x] Auth endpoints:
  - `POST /api/auth/register` (email + password)
  - `POST /api/auth/login` (email + password)
  - `POST /api/auth/google/login` (Google OAuth via axum-security OIDC)
  - `POST /api/auth/logout`
  - `GET /api/auth/me` (current user + roles)
  - `GET /api/auth/find-player` (find player to link)
  - `POST /api/auth/link-player` / `POST /api/auth/unlink-player`
- [x] JWT session management via axum-security
- [x] Role-based middleware via `#[requires]` / `#[requires_any]` macros

### Frontend
- [x] Initialize SvelteKit project with TypeScript, Tailwind CSS, Skeleton UI
- [x] PWA setup (vite-pwa plugin, manifest, service worker, icons)
- [x] Auth pages: login, register, Google OAuth button
- [x] Auth state management (`auth.svelte.ts` with user, token, roles)
- [x] Navigation layout: header with team name, nav links, login/avatar

### Verification
- [x] Register a user, log in, see `/me` return correct data
- [x] Google OAuth flow works end-to-end
- [x] PWA installable on mobile

---

## Phase 2: Players -- DONE

### Backend
- [x] Database schema: `players` table (id, user_id nullable, first_name, last_name, shirt_number, position, active, team_id, created_at)
  - Position enum with 10 positions (Keeper, Centre Back, Left/Right Back, etc.)
- [x] Endpoints:
  - `GET /api/players` (list all active players — public)
  - `GET /api/players/:id` (player detail — public)
  - `POST /api/players` (create — admin/moderator)
  - `PUT /api/players/:id` (update — admin/moderator)
  - `DELETE /api/players/:id` (soft delete via active=false — admin only)

### Frontend
- [x] Player list page (with toggle for inactive players for admin/mod)
- [x] Player detail/profile page (stats integrated from Phase 4)
- [x] Admin/moderator: player management UI (create, edit, deactivate)

### Verification
- [x] Create players, view player list as guest
- [x] Only admin/moderator can create/edit players

---

## Phase 3: Matches -- DONE

### Backend
- [x] Database schema: `games` table (id, opponent, location, date_time, home_away, status [scheduled/completed/cancelled], home_score, away_score, created_at)
- [x] Endpoints:
  - `GET /api/games` (list — public, ordered by date)
  - `GET /api/games/:id` (detail with events — public)
  - `POST /api/games` (create — admin/moderator)
  - `PUT /api/games/:id` (update — admin/moderator)
  - `DELETE /api/games/:id` (admin only)
  - `GET /api/games/upcoming`, `/api/games/recent`, `/api/games/summary`

### Frontend
- [x] Match list page with tabs: upcoming vs recent results
- [x] Match detail page (opponent, location, time, score, event timeline)
- [x] Admin/moderator: match management UI (create, edit, update score)

### Verification
- [x] Create a match, view it as guest
- [x] Update match with final score

---

## Phase 4: Match Events & Player Stats -- DONE

### Backend
- [x] Database schema: `game_events` table (id, game_id, player_id, event_type [goal/assist/yellow_card/red_card], minute, created_at)
- [x] Endpoints:
  - `GET /api/games/:id/events` (public)
  - `POST /api/games/:id/events` (admin/moderator)
  - `DELETE /api/games/:id/events/:event_id` (admin/moderator)
  - `GET /api/players/:id/stats` (aggregated stats with per-match breakdowns and cumulative timeline — public)
  - `GET /api/stats/leaderboard` (top scorers, top assists, most carded — public)

### Frontend
- [x] Match detail: event timeline with goals, cards, assists and minute
- [x] Player profile: aggregated stats (appearances, goals, assists, yellow/red cards) + charts (bar chart, cumulative timeline)
- [x] Leaderboard/stats overview page (top scorers, top assisters, most carded)

### Verification
- [x] Add goals/assists/cards to a completed match
- [x] Player stats page shows correct aggregated numbers including card counts
- [x] Leaderboard ranks players correctly

---

## Phase 5: Polish & PWA -- DONE

- [x] PWA manifest + service worker with static asset precaching
- [x] Responsive design with Tailwind (mobile-ready)
- [x] Loading states (via TanStack Query `isPending`/`isLoading`)
- [x] Error handling (try-catch in handlers, error messages displayed)
- [x] Empty states with Dutch messaging throughout
- [x] Basic SEO/meta tags (description, theme-color, favicon, manifest link)
- [x] Offline caching of API data — Workbox StaleWhileRevalidate for `/api/*` + TanStack Query persistence to IndexedDB
- N/A Dynamic per-page meta tags — skipped, app runs in SPA mode (`ssr = false`) so meta tags aren't visible to crawlers

---

## Phase 5.5: Player Self-Service -- DONE

### Backend
- [x] Allow authenticated users linked to a player to update their own shirt number and position
  - `PUT /api/players/:id` — extend existing endpoint: if the logged-in user's `player_id` matches `:id`, allow updating `shirt_number` and `position` (no admin/moderator required)
  - Admin/moderator can still update everything (name, active status, etc.)

### Frontend
- [x] Player profile page: show "Bewerken" (edit) button for the linked player (not just admin/moderator)
- [x] Edit form: players can only change shirt number and position, not name or active status

### Verification
- [x] A regular user linked to a player can update their own shirt number and position
- [x] A regular user cannot update another player's details
- [x] Admin/moderator can still update all fields for any player

---

## Phase 6: Live Match & Push Notifications -- DONE

> **Deployment note:** this app is deployed on **AWS Lambda**, which does not support long-lived connections (no SSE, no WebSockets). Live updates use **HTTP polling** from the client instead. Push notifications still work because Web Push is a fire-and-forget HTTP request from the server to the push service.

### Backend
- [x] Derived match status (`scheduled`/`live`/`finished`/`cancelled`) computed from `date_time` + 120-minute window
- [x] Polling endpoint: `GET /api/games/:id/live` — returns current score, events, status, and `version` for cheap change detection
  - ETag support (`W/"<id>-<version>"`) with `If-None-Match` / `304 Not Modified`
  - Version field incremented on every mutation (score change, event add/delete)
- [x] `POST /api/games/:id/live/opponent_score` — moderator quick-action to adjust opponent score (±1), triggers goal push on +1
- [x] Push notification integration:
  - Database schema: `push_subscriptions` table (id, user_id, endpoint, p256dh, auth, notify_goal, created_at)
  - `POST /api/push/subscriptions` — upsert subscription by endpoint
  - `DELETE /api/push/subscriptions` — unsubscribe
  - `GET /api/push/subscriptions/me` — list current user's subscriptions
  - `GET /api/push/vapid-public-key` — return server VAPID key
  - Web Push protocol (RFC 8030) with VAPID keys
  - Push triggered on goal events and opponent score increments during live mode
  - Expired endpoints pruned automatically (410/404/401)

### Frontend
- [x] Live match view: auto-updating score and event timeline via visibility-aware polling (3s visible, 30s hidden, immediate on refocus)
- [x] Moderator quick-actions: opponent score ±1 buttons visible during live matches
- [x] Visual indicator: pulsing "LIVE" badge on match detail when active
- [x] Service worker handles push events, displays goal notifications with game link
- [x] Profile page: push notification toggle with permission/support status display

### Not implemented (deferred)
- Manual start/end live mode toggle (auto-detection from kickoff + 120min works well for now)
- Match start/end push notifications (only goals trigger push)
- Granular per-type notification preferences UI (notify_goal field exists but no UI beyond on/off)

### Verification
- [x] Add a goal event during a live match, see it appear on another device within one poll interval
- [x] Receive push notification on mobile when a goal is scored
- [x] Polling pauses/slows when the tab is hidden and resumes on focus

---

## Phase 6.5: CI/CD -- DONE

- [x] GitHub Actions workflow (`.github/workflows/ci.yml`) on push to main + PRs
- [x] **Backend job:** `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` with PostgreSQL 16 service container
- [x] **Frontend job:** `prettier --check`, `svelte-check`, `vitest` with Node 22
- [x] Rust caching via `Swatinem/rust-cache`, npm caching via `actions/setup-node`
- [x] Per-test database isolation (unique PostgreSQL database per test via `tokio-postgres`)

---

## Phase 7: This Week's Match Highlight -- DONE

### Frontend
- [x] Wedstrijden tab: if there is a match this week, show it prominently at the top (highlighted card with opponent, date/time, location)
- [x] Clear visual distinction from the regular match list (accent left border, primary tonal background, "Deze week" label)
- [x] If the match is today or live, emphasize further ("Vandaag" badge or "LIVE" indicator with score)
- [x] Highlighted match removed from regular upcoming list to avoid duplication

### Verification
- [x] A match scheduled this week appears highlighted at the top of the wedstrijden tab
- [x] When no match this week, the tab shows the normal list without a highlight
- [x] A live match this week shows the live indicator with score in the highlight card

---

## Phase 8: Admin User Management -- DONE

### Backend
- [x] `GET /api/users` — list all users with roles (admin only)

### Frontend
- [x] Admin-only page at `/admin/users` listing all users (name, email, role chips)
- [x] Toggle buttons to promote/demote moderator role per user (uses existing `PATCH /api/users/:id`)
- [x] Nav link to user management visible to admins only

### Verification
- [x] Admin can view all users and their roles
- [x] Admin can toggle moderator role for any user
- [x] Non-admins cannot access the user management page

---

## Phase 9: Multi-Team Support -- NOT STARTED

> **Breaking change:** this phase replaces the single-team `opponent` + `home_away` model with explicit `home_team_id` / `away_team_id` foreign keys. A database migration is required. The `HomeAway` enum is removed.

### Backend — Schema

- [ ] Add `home_team_id: Uuid` and `away_team_id: Uuid` fields to `Game` model (both FK to `teams.id`)
- [ ] Remove `opponent: String` and `home_away: HomeAway` fields from `Game` model
- [ ] Remove `home_away.rs` module and all `HomeAway` references
- [ ] Write migration: add columns, backfill from `opponent` + `home_away` (create team rows for each distinct opponent, set home/away team IDs), then drop old columns

### Backend — Handlers & Logic

- [ ] Update `CreateGameRequest` / `UpdateGameRequest` — replace `opponent`/`homeAway` with `homeTeamId`/`awayTeamId`
- [ ] Eager-load home/away team names in `GameResponse` (avoid N+1)
- [ ] Rework `adjust_opponent_score` in `games/live.rs` — use team IDs instead of `HomeAway` enum
- [ ] Score increment logic in event handlers: determine side by checking `player.team_id` against `game.home_team_id` / `game.away_team_id`
- [ ] Add optional `?team_id=` query param to leaderboard endpoint for team filtering
- [ ] Update push notification payloads — use team names instead of opponent string
- [ ] Add `GET /api/teams` endpoint (public) for team selection in UI
- [ ] Update seed data to use team FKs instead of opponent strings

### Frontend — Types

- [ ] Remove `HomeAway` type
- [ ] Update `Game` interface — replace `opponent` + `homeAway` with `homeTeam: { id: string; name: string }` and `awayTeam: { id: string; name: string }`
- [ ] Update `CreateGameRequest` / `UpdateGameRequest` with `homeTeamId` / `awayTeamId`
- [ ] Add `Team` interface (`id: string`, `name: string`)

### Frontend — Pages & Components

- [ ] Game create/edit: replace opponent text input + thuis/uit radio with two team selectors (dropdowns from `GET /api/teams`)
- [ ] Match display: show both team names instead of `vs {opponent}`
- [ ] Event form: allow selecting players from either team
- [ ] Leaderboard: add optional team filter dropdown
- [ ] Player stats: update game timeline entries with both team names
- [ ] Push notification text: use team names instead of opponent string

### Configuration

- [ ] Add `OWN_TEAM_ID` env var to identify "our team" for UX shortcuts (opponent score adjuster, default leaderboard filter)

### Verification

- [ ] Create a match between two teams using the team selector UI
- [ ] Score display shows both team names correctly
- [ ] Goal events increment the correct side based on the player's team
- [ ] Opponent score adjuster works correctly during live matches
- [ ] Leaderboard shows stats for all players and filters by team
- [ ] Push notifications fire with correct team names
- [ ] Existing seed data migrates correctly

---

## Phase 10: Performance -- NOT STARTED

> **Goal:** reduce response times and payload sizes across the stack. Uses a Lambda-local in-memory cache (effective during warm container reuse, gracefully empty on cold starts) combined with query optimization and frontend bundle improvements.

### Backend — In-Memory Cache

- [ ] Add a process-local cache (e.g. `moka` or `mini-moka`) to `AppState` with TTL-based expiration
- [ ] Cache frequently read, rarely written data: team list, player list, leaderboard, recent/upcoming games
- [ ] Invalidate relevant cache entries on writes (game create/update, event add/delete, player update)
- [ ] Keep cache miss path identical to current behavior — cold starts just skip the cache

### Backend — Query Optimization

- [ ] Audit database queries for N+1 patterns — eager-load relations (teams on games, players on events) in list endpoints
- [ ] Add database indexes on hot query paths: `games(date_time)`, `game_events(game_id)`, `players(team_id)`, `game_events(player_id)`
- [ ] Optimize leaderboard query — aggregate in a single query instead of per-player lookups
- [ ] Review `stats` endpoint for redundant queries and consolidate where possible

### Backend — Response Optimization

- [ ] Enable response compression (gzip/brotli) via `tower-http` `CompressionLayer`
- [ ] Extend ETag / `304 Not Modified` support beyond live polling to game list and player list endpoints
- [ ] Trim unnecessary fields from list responses (e.g. game list doesn't need full event arrays)

### Frontend — Bundle & Runtime

- [ ] Audit bundle size — identify and tree-shake unused Skeleton UI components and dependencies
- [ ] Lazy-load route chunks for admin/moderator-only pages (game edit, player management)
- [ ] Optimize TanStack Query `staleTime` / `gcTime` settings per query to reduce unnecessary refetches
- [ ] Review service worker precache list — only precache critical assets, let non-critical assets use runtime caching
- [ ] Compress and resize any static image assets (icons, team logos)

### Verification

- [ ] Measure cold start and warm response times before and after cache implementation
- [ ] Confirm cache invalidation works: update a game, immediately fetch the list, see the change
- [ ] Verify `304 Not Modified` responses on unchanged resources reduce payload transfer
- [ ] Run Lighthouse audit on frontend — target performance score improvement
- [ ] Confirm no regressions in existing functionality after query/response changes

---

## Data Model Summary

```
users (id, email, password_hash, first_name, last_name, player_id, avatar_url, is_admin, is_moderator, created_at)
players (id, user_id?, first_name, last_name, shirt_number, position, active, team_id, created_at)
games (id, home_team_id, away_team_id, location, date_time, home_score, away_score, cancelled, version, updated_at, created_at)
game_events (id, game_id, player_id, event_type, minute, created_at)
push_subscriptions (id, user_id, endpoint, p256dh_key, auth_key, created_at)  -- Phase 6
```

## Suggested Implementation Order

1. ~~Phase 1 — Scaffolding & Auth~~ DONE
2. ~~Phase 2 — Players~~ DONE
3. ~~Phase 3 — Matches~~ DONE
4. ~~Phase 4 — Match Events & Stats~~ DONE
5. ~~Phase 5 — Polish & PWA enhancements~~ DONE
6. ~~Phase 5.5 — Player self-service (users update own shirt number & position)~~ DONE
7. ~~Phase 6 — Live match mode with polling + push notifications on goals~~ DONE
8. ~~Phase 6.5 — CI/CD with GitHub Actions~~ DONE
9. ~~Phase 7 — This week's match highlight in wedstrijden tab~~ DONE
10. ~~Phase 8 — Admin user management UI~~ DONE
11. Phase 9 — Multi-team support: explicit home/away team FKs, team-aware events & stats
12. Phase 10 — Performance: Lambda-local cache, query optimization, compression, frontend bundle

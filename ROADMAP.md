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

> **Deployment note:** this app is deployed on **Fly.io** as a persistent Axum process. Live updates use per-match **WebSockets** at `/api/games/:id/ws`. Push notifications use Web Push for OS-level alerts when the app is closed/backgrounded.

### Backend
- [x] Derived match status (`scheduled`/`live`/`finished`/`cancelled`) computed from `date_time` + 120-minute window
- [x] WebSocket endpoint: `GET /api/games/:id/ws` — streams `Snapshot` / `ScoreUpdate` / `EventAdded` / `EventDeleted` / `StatusChange` frames
- [x] In-memory `LiveHub` that broadcasts events to all connected sockets for a given match
- [x] Legacy polling endpoint: `GET /api/games/:id/live` with ETag / `304 Not Modified` support (kept as fallback)
- [x] `POST /api/games/:id/live/score` — moderator quick-action to adjust score by side (±1), triggers goal push on +1
- [x] Push notification integration:
  - Database schema: `push_subscriptions` table (id, user_id, endpoint, p256dh, auth, notify_goal, created_at)
  - `POST /api/push/subscriptions` — upsert subscription by endpoint
  - `DELETE /api/push/subscriptions` — unsubscribe
  - `GET /api/push/subscriptions/me` — list current user's subscriptions
  - `GET /api/push/vapid-public-key` — return server VAPID key
  - Web Push protocol (RFC 8030) with VAPID keys
  - Push triggered on goal events and score increments during live mode
  - Expired endpoints pruned automatically (410/404/401)

### Frontend
- [x] Live match view: real-time score and event timeline via WebSocket with auto-reconnect (exponential backoff 1s → 30s)
- [x] Visibility-aware: disconnects after 60s of tab being hidden, reconnects on focus
- [x] Moderator quick-actions: ±1 score buttons per side visible during live matches
- [x] Visual indicator: pulsing "LIVE" badge on match detail when active
- [x] Service worker handles push events, displays goal notifications with game link
- [x] Profile page: push notification toggle with permission/support status display

### Not implemented (deferred)
- Manual start/end live mode toggle (auto-detection from kickoff + 120min works well for now)
- Match start/end push notifications (only goals trigger push)
- Granular per-type notification preferences UI (`notify_goal` field exists but no UI beyond on/off)

### Verification
- [x] Add a goal event during a live match, see it appear on another device instantly via WebSocket
- [x] Receive push notification on mobile when a goal is scored
- [x] WebSocket disconnects when tab is hidden and reconnects on focus

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

## Phase 9: Multi-Team Support -- DONE

> **Breaking change:** this phase replaced the single-team `opponent` + `home_away` model with explicit `home_team_id` / `away_team_id` foreign keys. The `HomeAway` enum is removed. "Own side" is derived per-user from their linked player's team (stored in JWT `team_id` claim), not from an env var.

### Backend — Schema

- [x] Add `home_team_id: Uuid` and `away_team_id: Uuid` fields to `Game` model (both FK to `teams.id`)
- [x] Remove `opponent: String` and `home_away: HomeAway` fields from `Game` model
- [x] Remove `home_away.rs` module and all `HomeAway` references
- [x] Update seed data to use team FKs instead of opponent strings

### Backend — Handlers & Logic

- [x] Update `CreateGameRequest` / `UpdateGameRequest` — replace `opponent`/`homeAway` with `homeTeamId`/`awayTeamId`
- [x] Eager-load home/away team names in `GameResponse` (avoid N+1)
- [x] Replace `adjust_opponent_score` with explicit-side `adjust_score` (`POST /{id}/live/score` with `{ side, delta }`)
- [x] Score increment logic in event handlers: determine side by checking `player.team_id` against `game.home_team_id` / `game.away_team_id`
- [x] Update push notification payloads — use team names instead of opponent string
- [x] Add `GET /api/teams` (public) and `POST /api/teams` (admin) endpoints
- [x] Add `team_id` to JWT claims for per-user own-side derivation

### Frontend — Types

- [x] Remove `HomeAway` type
- [x] Update `Game` interface — replace `opponent` + `homeAway` with `homeTeam` / `awayTeam` objects
- [x] Update `CreateGameRequest` / `UpdateGameRequest` with `homeTeamId` / `awayTeamId`
- [x] Add `Team` interface and `ScoreSide` type

### Frontend — Pages & Components

- [x] Game create/edit: two team dropdowns populated from `GET /api/teams`, inline team creation
- [x] Match display: `{homeTeam.name} vs {awayTeam.name}` on list, detail, and home pages
- [x] Own-side (thuis/uit) chip derived per-user from `auth.teamId`
- [x] Score adjuster: opponent-only panel when `ownSide !== null`, both-side controls for non-team moderators
- [x] Header sidebar subtitle shows both team names for today's game
- [x] Push notification text uses team names
- [x] All frontend tests updated for new Game shape

### Verification

- [x] `cargo fmt --check && cargo clippy -- -D warnings && cargo test` — all green
- [x] `npx svelte-check && npx vitest run` — 0 errors, all tests pass
- [x] Create a match between two teams using the team selector UI
- [x] Score display shows both team names correctly
- [x] Goal events increment the correct side based on the player's team
- [x] Score adjuster works correctly during live matches
- [x] Push notifications fire with correct team names

---

## Phase 10: Performance -- NOT STARTED

> **Goal:** reduce response times and payload sizes across the stack. Since we run on Fly.io as a persistent process, a process-local in-memory cache (e.g. `moka`) stays warm across requests and is the right tool here.

### Backend — In-Memory Cache

- [ ] Add a process-local cache (e.g. `moka` or `mini-moka`) to `AppState` with TTL-based expiration
- [ ] Cache frequently read, rarely written data: team list, player list, leaderboard, recent/upcoming games
- [ ] Invalidate relevant cache entries on writes (game create/update, event add/delete, player update)

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

---

## Phase 11: Live Tab Cleanup -- NOT STARTED

> Small but impactful refinements to the live match view. Items from the backlog: own goals, optional player on score adjustments, assist grouping, and notification smoke-testing.

### Backend
- [ ] Add `OwnGoal` variant to `EventType` enum (variant 5) — increments the *opposing* team's score when added as an event, so moderators don't need the ±1 score adjuster for own goals
- [ ] Allow `player_id` to be `null` on `game_events` for own goals or anonymous score corrections

### Frontend
- [ ] Surface `own_goal` as a selectable event type in the add-event form ("Eigen doelpunt") with its own icon
- [ ] Group assists with their adjacent goal in the event timeline: show assist indented under the goal it belongs to (same minute, different player), rather than as a separate standalone row
- [ ] Notification mock: add a button in the profile page (visible only in dev/staging, or gated behind a flag) that triggers a test push notification so moderators can verify their subscription is working without waiting for a real goal
- [ ] Google OAuth integration test: add at least a CI smoke-test that walks through the OIDC redirect flow using a mock identity provider — the happy path is wired up but never tested in CI

### Verification
- [ ] Add an own-goal event; confirm it increments the opponent's score, not the player's team
- [ ] Add a goal + assist at the same minute; confirm assist is grouped under the goal in the UI
- [ ] Trigger test notification from profile page; confirm OS notification appears on device

---

## Phase 12: Player List UX -- DONE

> Sort the players page by position so that the most forward-facing positions appear first, matching how a football lineup is typically read.

### Frontend
- [x] Define a position sort order (Spits → Linksvleugel → Rechtsvleugel → Aanvallende mid → Centrale mid → Defensieve mid → Linksback → Rechtsback → Centrale verdediger → Keeper)
- [x] Sort `players` list by this order on the players page; within the same position sort alphabetically by last name
- [x] Group by position with Dutch headers between groups ("Aanvallers", "Middenvelders", "Verdedigers", "Keeper")

### Verification
- [x] Players page shows forwards at the top and keeper at the bottom
- [x] Within each position group, players are sorted alphabetically
- [x] 5 component tests covering group headers, sort order, alpha sort, loading state, and empty state

---

## Data Model Summary

```
users        (id, email, password_hash, first_name, last_name, player_id, avatar_url, is_admin, is_moderator, created_at)
teams        (id, name, created_at)
players      (id, user_id?, first_name, last_name, shirt_number, position, active, team_id, created_at)
games        (id, home_team_id, away_team_id, location, date_time, home_score, away_score, cancelled, version, updated_at, created_at)
game_events  (id, game_id, player_id?, event_type, minute, created_at)  -- player_id nullable after Phase 11
push_subscriptions (id, user_id, endpoint, p256dh_key, auth_key, created_at)
```

> `avatar_url` on users is already fully implemented: multipart upload endpoint, 256×256 WebP resize, Google OAuth seeds it on first login, `PlayerAvatar` component used throughout the frontend.

## Suggested Implementation Order

1. ~~Phase 1 — Scaffolding & Auth~~ DONE
2. ~~Phase 2 — Players~~ DONE
3. ~~Phase 3 — Matches~~ DONE
4. ~~Phase 4 — Match Events & Stats~~ DONE
5. ~~Phase 5 — Polish & PWA enhancements~~ DONE
6. ~~Phase 5.5 — Player self-service (users update own shirt number & position)~~ DONE
7. ~~Phase 6 — Live match mode with WebSockets + push notifications on goals~~ DONE
8. ~~Phase 6.5 — CI/CD with GitHub Actions~~ DONE
9. ~~Phase 7 — This week's match highlight in wedstrijden tab~~ DONE
10. ~~Phase 8 — Admin user management UI~~ DONE
11. ~~Phase 9 — Multi-team support: explicit home/away team FKs, team-aware events & stats~~ DONE
12. Phase 11 — Live tab cleanup: own goals, assist grouping, notification mock, Google OAuth CI test
13. ~~Phase 12 — Player list UX: sort by position (strikers first), position group headers~~ DONE
14. Phase 10 — Performance: process-local cache, query optimization, compression, frontend bundle

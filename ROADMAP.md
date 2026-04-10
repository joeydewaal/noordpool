# Noordpool - Football Team App Roadmap

## Context

A PWA for a single football team where players can view upcoming matches, match results (with goal scorers, assists, cards, timestamps), and personal stats. Guest users can view everything read-only. Authenticated users have roles: **Admin** (full control including user/role management), **Moderator** (manage matches & player stats), **Player** (view only). Users can have multiple roles.

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

## Phase 7: Admin User Management -- NOT STARTED

### Backend
- [ ] `GET /api/users` — list all users with roles and linked player (admin only)

### Frontend
- [ ] Admin-only page at `/admin/users` listing all users in a table (name, email, roles, linked player)
- [ ] Toggle buttons to promote/demote moderator role per user (uses existing `PATCH /api/users/:id`)
- [ ] Nav link to user management visible to admins only

### Verification
- [ ] Admin can view all users and their roles
- [ ] Admin can toggle moderator role for any user
- [ ] Non-admins cannot access the user management page

---

## Data Model Summary

```
users (id, email, password_hash, first_name, last_name, player_id, avatar_url, is_admin, is_moderator, created_at)
players (id, user_id?, first_name, last_name, shirt_number, position, active, team_id, created_at)
games (id, opponent, location, date_time, home_away, status, home_score, away_score, created_at)
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
9. Phase 7 — Admin user management UI

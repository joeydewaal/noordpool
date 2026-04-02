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

## Phase 5: Polish & PWA -- PARTIAL

- [x] PWA manifest + service worker with static asset precaching
- [x] Responsive design with Tailwind (mobile-ready)
- [x] Loading states (via TanStack Query `isPending`/`isLoading`)
- [x] Error handling (try-catch in handlers, error messages displayed)
- [x] Empty states with Dutch messaging throughout
- [x] Basic SEO/meta tags (description, theme-color, favicon, manifest link)
- [ ] Offline caching of API data (match schedule, recent results) — only static assets cached, no IndexedDB/API cache strategy
- [ ] Dynamic per-page meta tags and Open Graph tags for social sharing

---

## Phase 6: Live Match & Push Notifications (Nice-to-have) -- NOT STARTED

### Backend
- [ ] Add `live` to match status enum (scheduled/live/completed/cancelled)
- [ ] `POST /api/matches/:id/live/start` — moderator starts live mode for a match
- [ ] `POST /api/matches/:id/live/end` — moderator ends live mode
- [ ] Server-Sent Events (SSE) endpoint: `GET /api/matches/:id/live/stream` — clients subscribe to real-time match events
  - SSE is simpler than WebSockets and sufficient since updates flow one-way (server to client)
  - Events pushed: goal scored, card given, substitution, score update, match ended
- [ ] Push notification integration:
  - Database schema: `push_subscriptions` table (id, user_id, endpoint, p256dh_key, auth_key, created_at)
  - `POST /api/push/subscribe` — register a device for push notifications
  - `DELETE /api/push/subscribe` — unsubscribe
  - Use Web Push protocol (RFC 8030) with VAPID keys
  - Trigger push notifications when a goal is scored or match ends during live mode

### Frontend
- [ ] Live match view: auto-updating score, event timeline that updates in real-time via SSE
- [ ] Visual indicator on match list when a match is live
- [ ] Service worker handles incoming push notifications and displays them
- [ ] Notification permission prompt in user settings or on first login
- [ ] Settings page: toggle which notifications to receive (goals, match start/end)

### Verification
- [ ] Start live mode, add a goal event, see it appear in real-time on another device
- [ ] Receive push notification on mobile when a goal is scored
- [ ] SSE reconnects gracefully if connection drops

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
5. Phase 5 — Polish & PWA enhancements (partially done, remaining: offline API caching, dynamic meta tags)
6. Phase 6 — Live match updates & push notifications (nice-to-have, not started)

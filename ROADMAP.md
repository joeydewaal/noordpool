# Noordpool - Football Team App Roadmap

## Context

A PWA for a single football team where players can view upcoming matches, match results (with goal scorers, assists, cards, timestamps), and personal stats. Guest users can view everything read-only. Authenticated users have roles: **Admin** (full control including user/role management), **Moderator** (manage matches & player stats), **Player** (view only). Users can have multiple roles.

## Tech Stack

- **Frontend:** SvelteKit (PWA)
- **Backend:** Rust with Axum, axum-security (local path dep), toasty (local path dep)
- **Database:** PostgreSQL
- **Auth:** Guest access (read-only), email+password, Google OAuth

---

## Phase 1: Project Scaffolding & Auth

### Backend
- Initialize Rust workspace with Axum
- Configure path dependencies for `axum-security` and `toasty`
- Set up PostgreSQL connection (via toasty or sqlx — depends on what toasty provides)
- Database schema: `users` table (id, email, password_hash, name, avatar_url, provider, created_at)
- Database schema: `roles` table or enum-based role assignment (admin, moderator, player) — many-to-many with users
- Auth endpoints:
  - `POST /api/auth/register` (email + password)
  - `POST /api/auth/login` (email + password)
  - `POST /api/auth/google` (Google OAuth callback)
  - `POST /api/auth/logout`
  - `GET /api/auth/me` (current user + roles)
- Session/token management via axum-security
- Role-based middleware for protected routes

### Frontend
- Initialize SvelteKit project
- PWA setup (service worker, manifest.json, offline support)
- Auth pages: login, register, Google OAuth flow
- Auth state management (store current user + roles)
- Navigation layout: header with team name, nav links, login/avatar

### Verification
- Register a user, log in, see `/me` return correct data
- Google OAuth flow works end-to-end
- PWA installable on mobile

---

## Phase 2: Players

### Backend
- Database schema: `players` table (id, user_id nullable, name, shirt_number, position, active)
  - `user_id` is nullable so players can exist without an account
- Endpoints:
  - `GET /api/players` (list all players — public)
  - `GET /api/players/:id` (player detail — public)
  - `POST /api/players` (create — admin/moderator)
  - `PUT /api/players/:id` (update — admin/moderator)
  - `DELETE /api/players/:id` (soft delete — admin only)

### Frontend
- Player list page
- Player detail/profile page (stats come later in Phase 4)
- Admin/moderator: player management UI (add, edit, deactivate)

### Verification
- Create players, view player list as guest
- Only admin/moderator can create/edit players

---

## Phase 3: Matches

### Backend
- Database schema: `matches` table (id, opponent, location, date_time, home_away, status [scheduled/completed/cancelled], home_score, away_score, created_at)
- Endpoints:
  - `GET /api/matches` (list — public, with filters: upcoming/past)
  - `GET /api/matches/:id` (detail — public)
  - `POST /api/matches` (create — admin/moderator)
  - `PUT /api/matches/:id` (update — admin/moderator)
  - `DELETE /api/matches/:id` (admin only)

### Frontend
- Match list page with tabs/filter: upcoming vs past results
- Match detail page (opponent, location, time, score)
- Admin/moderator: match management UI (create, edit, update score)

### Verification
- Create a match, view it as guest
- Update match with final score

---

## Phase 4: Match Events & Player Stats

### Backend
- Database schema: `match_events` table (id, match_id, player_id, event_type [goal/assist/yellow_card/red_card], minute, created_at)
- Endpoints:
  - `GET /api/matches/:id/events` (public)
  - `POST /api/matches/:id/events` (admin/moderator)
  - `PUT /api/matches/:id/events/:event_id` (admin/moderator)
  - `DELETE /api/matches/:id/events/:event_id` (admin/moderator)
  - `GET /api/players/:id/stats` (aggregated stats — public)
  - `GET /api/stats/leaderboard` (top scorers, top assists — public)
### Frontend
- Match detail: show timeline of events (goals, cards with minute)
- Player profile: show aggregated stats (appearances, goals, assists, yellow cards, red cards)
- Leaderboard/stats overview page (include a "most carded" section)

### Verification
- Add goals/assists/cards to a completed match
- Player stats page shows correct aggregated numbers including card counts
- Leaderboard ranks players correctly

---

## Phase 5: Polish & PWA

- Offline caching of match schedule and recent results
- Responsive design tuning for mobile
- Loading states, error handling, empty states
- SEO/meta tags for shared links

---

## Phase 6: Live Match & Push Notifications (Nice-to-have)

### Backend
- Add `live` to match status enum (scheduled/live/completed/cancelled)
- `POST /api/matches/:id/live/start` — moderator starts live mode for a match
- `POST /api/matches/:id/live/end` — moderator ends live mode
- Server-Sent Events (SSE) endpoint: `GET /api/matches/:id/live/stream` — clients subscribe to real-time match events
  - SSE is simpler than WebSockets and sufficient since updates flow one-way (server to client)
  - Events pushed: goal scored, card given, substitution, score update, match ended
- Push notification integration:
  - Database schema: `push_subscriptions` table (id, user_id, endpoint, p256dh_key, auth_key, created_at)
  - `POST /api/push/subscribe` — register a device for push notifications
  - `DELETE /api/push/subscribe` — unsubscribe
  - Use Web Push protocol (RFC 8030) with VAPID keys
  - Trigger push notifications when a goal is scored or match ends during live mode

### Frontend
- Live match view: auto-updating score, event timeline that updates in real-time via SSE
- Visual indicator on match list when a match is live
- Service worker handles incoming push notifications and displays them
- Notification permission prompt in user settings or on first login
- Settings page: toggle which notifications to receive (goals, match start/end)

### Verification
- Start live mode, add a goal event, see it appear in real-time on another device
- Receive push notification on mobile when a goal is scored
- SSE reconnects gracefully if connection drops

---

## Data Model Summary

```
users (id, email, password_hash, name, avatar_url, provider, created_at)
user_roles (user_id, role) -- role: admin | moderator | player
players (id, user_id?, name, shirt_number, position, active)
matches (id, opponent, location, date_time, home_away, status, home_score, away_score)
match_events (id, match_id, player_id, event_type, minute)
push_subscriptions (id, user_id, endpoint, p256dh_key, auth_key, created_at)  -- Phase 6
```

## Suggested Implementation Order

1. Phase 1 — Scaffolding & Auth (start here)
2. Phase 2 — Players
3. Phase 3 — Matches
4. Phase 4 — Match Events & Stats
5. Phase 5 — Polish & PWA enhancements
6. Phase 6 — Live match updates & push notifications (nice-to-have)

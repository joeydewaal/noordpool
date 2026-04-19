# Game Lineup — Design Document

## Overview

A FIFA-style team lineup view attached to each game. Anyone can view the
lineup; only admins/moderators can edit it. Editors pick a formation (4-4-2,
4-3-3, …), drag players onto the pitch, and assign the rest to the bench.
Each new game pre-loads the previous game's lineup so you never start from
scratch. On mobile the interaction is drag-and-drop with touch support.
Live updates (Stage 4) push changes to all viewers over the existing WebSocket.

---

## Stages

| Stage | Scope | Deliverable |
|-------|-------|-------------|
| **1** | Backend + read-only UI | Models, migrations, GET/PUT API, pitch view with static badges, link from game detail |
| **2** | Edit mode (click, no drag) | Formation switcher, click-slot → player picker, bench, Opslaan, previous-lineup pre-load |
| **3** | Drag & drop | `svelte-dnd-action` for desktop + touch, swap on drop |
| **4** | Live updates | `LineupUpdated` WS event, all viewers see changes in real time |
| **5** | Extras | Captain marker, player availability, lineup push notification, share-as-image |

---

## UX

### Layout — desktop

```
┌──────────────────────────────────────────────────┐
│  Opstelling  [4-4-2][4-3-3][4-2-3-1][3-5-2]…   │
├──────────────────────┬───────────────────────────┤
│                      │  Bank (7 max)             │
│   [badge] [badge]    │  [badge][badge][badge]→   │
│  [b][b][b][b]        │                           │
│  [b][b][b][b]        │  Niet ingepland           │
│       [GK]           │  [b][b][b]…               │
│                      │                           │
│         pitch        │  [ Opslaan ]  (mod only)  │
└──────────────────────┴───────────────────────────┘
```

### Layout — mobile

```
┌─────────────────────┐
│ [4-4-2][4-3-3][…]  │  ← scrollable pill tabs
├─────────────────────┤
│                     │
│  [badge]  [badge]   │  ← rows of badges, full-width pitch
│ [b][b][b][b]        │
│ [b][b][b][b]        │
│      [GK]           │
│                     │
├─────────────────────┤
│ Bank  [b][b][b]  →  │  ← horizontal scroll
└─────────────────────┘
```

Tap empty slot → bottom sheet with unassigned player list.
Tap filled slot (mod) → contextual menu: swap, move to bench, remove.
Drag (Stage 3) → long-press to pick up, drag to target slot or bench.

---

## Player Badge — dark FIFA card style

The badge mimics a FIFA Ultimate Team card: cut top corners, dark charcoal
background, player photo dominant, no stat numbers.

```
    ╱‾‾‾‾‾‾‾‾‾‾╲
   │  9      GK  │   ← shirt number (top-left), slot role (top-right), small text
   │  ┌────────┐ │
   │  │  📸    │ │   ← avatar (round crop, ~48 px) or initials fallback
   │  └────────┘ │
   │  Jan de     │
   │  Boer       │   ← name, two lines max, truncated
    ╲____________╱
```

**Visual spec:**
- Shape: `clip-path: polygon(12% 0%, 88% 0%, 100% 8%, 100% 100%, 0% 100%, 0% 8%)`
  gives the characteristic diagonal-cut top corners.
- Size: 72 × 92 px desktop, 58 × 74 px mobile.
- Background: dark gradient `#1c1c28 → #111118`, slight top-to-bottom lightening.
- Thin border: `1px solid rgba(255,255,255,0.08)` for depth.
- Subtle inner glow or metallic sheen on the border using `box-shadow`.
- Text: white. Shirt number + role: `font-weight: 700`, slightly larger.
- Photo: circular crop centered in the upper ⅔ of the card.
- If no avatar: show shirt number in a large circle as the photo placeholder
  (same style as the existing `PlayerAvatar` component).
- Empty slot: same shape, background `rgba(255,255,255,0.04)`, dashed border,
  role label centred in dimmed text. Clicking (mod) opens the player picker.

---

## Previous Lineup Pre-load

When a moderator opens the lineup for game G with no saved lineup yet, the
backend `GET /api/games/:id/lineup` auto-seeds from the most recent
*completed* game involving the same home team:

```
GET /api/games/:id/lineup
  → if GameLineup exists for game G: return it
  → else: find latest GameLineup where game.homeTeamId = G.homeTeamId
          copy formation + slots into a new GameLineup for G (unsaved draft)
          return it with a `seededFrom: <game_id>` field
  → else: return null (empty pitch)
```

The response includes `"seededFrom": "<previous-game-id>"` when pre-loaded
so the frontend can show a banner: *"Opstelling overgenomen van [date]."*

The draft is NOT written to the database until the moderator hits Opslaan —
this way the lineup only exists once intentionally published.

---

## Formations

Formations are pure frontend constants — no backend storage needed.

```ts
interface FormationSlot {
  role: "GK" | "DEF" | "MID" | "FWD";
  x: number; // % from left edge of pitch
  y: number; // % from top (opponent goal = 0%, own goal = 100%)
}

interface Formation {
  id: string;          // "4-4-2"
  label: string;
  slots: FormationSlot[]; // always length 11, index 0 = GK
}
```

Initial set: **4-4-2**, **4-3-3**, **4-2-3-1**, **3-5-2**, **5-3-2**, **4-1-4-1**.

When switching formations the frontend remaps slot assignments: players in
slots that still exist (same index) stay put; extras are moved to bench.

Slot coordinates for 4-4-2 (portrait):

| slot | role | x%  | y%  |
|------|------|-----|-----|
| 0    | GK   | 50  | 88  |
| 1–4  | DEF  | 18/38/62/82 | 70 |
| 5–8  | MID  | 18/38/62/82 | 50 |
| 9    | FWD  | 33  | 28  |
| 10   | FWD  | 67  | 28  |

---

## Data Model

### `GameLineup`

```rust
pub struct GameLineup {
    #[key] #[auto]
    pub id: Uuid,
    #[index]
    pub game_id: Uuid,
    #[belongs_to(key = game_id, references = id)]
    pub game: BelongsTo<Game>,
    pub formation: String,       // "4-4-2"
    pub published: bool,         // false = draft, true = visible to all
    pub updated_at: Timestamp,
}
```

`published: false` lets mods draft changes without immediately showing them
to viewers. Hitting "Publiceer" flips this flag and (Stage 4) broadcasts a
`LineupUpdated` WS event.

### `GameLineupSlot`

```rust
pub struct GameLineupSlot {
    #[key] #[auto]
    pub id: Uuid,
    pub lineup_id: Uuid,
    #[belongs_to(key = lineup_id, references = id)]
    pub lineup: BelongsTo<GameLineup>,
    pub player_id: Uuid,
    #[belongs_to(key = player_id, references = id)]
    pub player: BelongsTo<Player>,
    /// 0–10 = starting XI slot, 11–17 = bench.
    pub slot: i32,
    /// True if this player is the captain.
    pub captain: bool,
}
```

### API response

```json
{
  "id": "…",
  "gameId": "…",
  "formation": "4-4-2",
  "published": true,
  "updatedAt": "…",
  "seededFrom": "prev-game-id-or-null",
  "slots": [
    {
      "slot": 0,
      "captain": false,
      "player": {
        "id": "…", "firstName": "Jan", "lastName": "de Boer",
        "shirtNumber": 1, "position": "Keeper",
        "user": { "avatarUrl": "…" }
      }
    }
  ]
}
```

---

## API

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `GET` | `/api/games/:id/lineup` | — | Fetch published lineup (or draft if mod) |
| `PUT` | `/api/games/:id/lineup` | Admin / Mod | Full replace, returns updated lineup |
| `POST` | `/api/games/:id/lineup/publish` | Admin / Mod | Flip `published = true`, broadcast WS event |

**Why separate publish?** Lets mods save a draft multiple times without
notifying players each time, then do a single intentional publish.

### `PUT` body

```json
{
  "formation": "4-4-2",
  "slots": [
    { "slot": 0, "playerId": "…", "captain": false },
    { "slot": 11, "playerId": "…", "captain": false }
  ]
}
```

Full-replace semantics: delete existing slots, insert new ones atomically.

---

## Live Updates — Stage 4

Reuse the existing per-game WebSocket at `/api/games/:id/ws`.

Add a new `LiveEvent` variant:

```rust
LineupUpdated {
    formation: String,
    slots: Vec<LineupSlotSummary>,
}
```

The `publish` handler broadcasts this after flipping `published = true`.
All connected clients receive it and update their local lineup state without
a full refetch. Non-published saves do NOT broadcast.

Frontend handler:

```ts
onLineupUpdated: ({ formation, slots }) => {
  lineupOverlay = { formation, slots };
}
```

---

## Frontend Architecture

### Route

`/games/:id/lineup` — separate sub-route. The pitch needs most of the
viewport and doesn't fit in the existing game card layout. The game detail
page gets a "Opstelling →" link in the header row.

### Data fetching

```ts
const lineupQuery = createQuery(() => ({
  queryKey: ["lineup", gameId],
  queryFn: () => getGameLineup(gameId),
}));

const saveLineupMutation = createMutation(() => ({
  mutationFn: (body: SaveLineupRequest) => putGameLineup(gameId, body),
  onSuccess: () => queryClient.invalidateQueries({ queryKey: ["lineup", gameId] }),
}));

const publishMutation = createMutation(() => ({
  mutationFn: () => publishLineup(gameId),
  onSuccess: () => queryClient.invalidateQueries({ queryKey: ["lineup", gameId] }),
}));
```

Local edit state (formation + slots) lives in `$state`, initialized from
`lineupQuery.data`. Mods see two buttons: **Opslaan** (saves draft) and
**Publiceer** (saves + publishes + notifies).

### Drag and drop — Stage 3

**`svelte-dnd-action`** (MIT, ~6 kB gzip, touch events built-in).

Three drop zones: pitch slots, bench, unassigned pool. Dropping onto an
occupied pitch slot swaps the two players. The pitch uses absolute
positioning for badges; `dndzone` wraps the entire pitch surface.

```svelte
<div
  use:dndzone={{ items: pitchSlots, type: "player" }}
  on:consider={handleConsider}
  on:finalize={handleFinalize}
  class="relative w-full h-full"
>
  {#each pitchSlots as slot}
    <div style="left: {slot.x}%; top: {slot.y}%;" class="absolute -translate-x-1/2 -translate-y-1/2">
      <PlayerBadge {slot} editable={canManage} />
    </div>
  {/each}
</div>
```

### Components

```
src/routes/games/[id]/lineup/
  +page.svelte          — route, owns edit state, fetches data
  PitchView.svelte      — SVG pitch background + absolutely positioned badges
  FormationTabs.svelte  — pill tabs, emits formation change
  PlayerBadge.svelte    — dark FIFA card (avatar, name, shirt #, role, captain C)
  SlotPlaceholder.svelte — empty slot (dashed, shows role label, clickable)
  BenchRow.svelte        — horizontal scroll, dnd zone, max 7
  PlayerPool.svelte      — unassigned players; bottom-sheet on mobile
  PlayerPickerSheet.svelte — modal/sheet for tap-to-assign flow (Stage 2)
```

---

## Permission Model

- `GET /api/games/:id/lineup` — public, returns lineup only if `published = true`.
  If the requester is admin/mod, return regardless of `published`.
- `PUT /api/games/:id/lineup` — admin/mod only.
- `POST /api/games/:id/lineup/publish` — admin/mod only.
- Frontend: drag handles, slot click, Opslaan/Publiceer only rendered for
  `auth.isAdmin || auth.isModerator`.

---

## Missing Features — Considered

| Feature | Include? | Notes |
|---------|----------|-------|
| **Captain marker** | ✅ Stage 2 | "C" badge on card, stored on `GameLineupSlot.captain` |
| **Draft vs. published** | ✅ Stage 2 | Mods can save drafts; Publiceer makes it visible + sends WS event |
| **Previous lineup pre-load** | ✅ Stage 2 | Seed from last game's lineup, shown as banner |
| **Player availability** | ✅ Stage 2 | Mark players absent/injured in the pool before assigning; stored on the slot or a separate `availability` flag on the player per game |
| **Out-of-position indicator** | ✅ Stage 2 | Yellow dot if `slot.role` doesn't match `player.position` group |
| **Lineup publish notification** | ✅ Stage 5 | Push notification to all subscribed players when Publiceer is clicked |
| **Live updates for viewers** | ✅ Stage 4 | WS `LineupUpdated` event |
| **Share as image** | Stage 5 | Server-side or canvas-based OG image for WhatsApp sharing |
| **Opposition formation** | Deferred | Show opponent's expected setup on top half of pitch |
| **Substitution plan** | Deferred | Pre-plan which sub replaces which player at what minute |
| **Lineup history** | Deferred | Browse past lineups per team |
| **Auto-suggest** | Deferred | Suggest lineup based on most-played positions from game events |
| **Multiple saved templates** | Deferred | Named presets per team ("Standaard 4-4-2", "Pressing 4-3-3") |
| **Tactical annotations** | Deferred | Draw arrows/zones on the pitch |
| **Team colour theming** | Deferred | Add `color` field to `Team`, use it for badge gradient |

---

## Open Decisions

| Question | Decision |
|----------|----------|
| Bench size | Fixed 7 (regulation max substitutes) |
| Formation change behaviour | Keep by slot index where possible, bench extras |
| Empty lineup state | Show empty pitch with slot placeholders, not a "no lineup" message |
| Draft visibility | Only admin/mod see unpublished lineup; public sees last published state |
| Pre-load source | Previous game by same home team (not away, since home team = Noordpool) |

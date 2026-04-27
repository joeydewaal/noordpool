export type Role = "admin" | "moderator" | "player";

export interface User {
  id: string;
  email: string;
  firstName: string;
  lastName: string;
  avatarUrl: string | null;
  playerId: string | null;
  isAdmin?: boolean;
  isModerator?: boolean;
  roles: Role[];
}

export interface LoginRequest {
  email: string;
  password: string;
}

export interface RegisterRequest {
  firstName: string;
  lastName: string;
  email: string;
  password: string;
}

export interface PlayerMatch {
  id: string;
  firstName: string;
  lastName: string;
  shirtNumber: number;
  position: Position;
}

export interface AuthResponse {
  user: User;
  token: string;
}

export type Position =
  | "Keeper"
  | "Centrale verdediger"
  | "Linksback"
  | "Rechtsback"
  | "Defensieve middenvelder"
  | "Centrale middenvelder"
  | "Aanvallende middenvelder"
  | "Linksvleugel"
  | "Rechtsvleugel"
  | "Spits";

export interface Player {
  id: string;
  userId: string | null;
  firstName: string;
  lastName: string;
  shirtNumber: number;
  position: Position;
  active: boolean;
  teamId: string;
  user?: User | null;
}

export interface CreatePlayerRequest {
  firstName: string;
  lastName: string;
  shirtNumber: number;
  position: Position;
  teamId: string;
}

export interface UpdatePlayerRequest {
  firstName?: string;
  lastName?: string;
  shirtNumber?: number;
  position?: Position;
  active?: boolean;
}

export interface Team {
  id: string;
  name: string;
}

/// Server-derived game status. Computed from `dateTime` + a fixed match
/// window on the backend; the frontend never derives liveness itself.
export type GameStatus = "scheduled" | "live" | "finished" | "cancelled";
export type ScoreSide = "home" | "away";

export interface GamesPage {
  items: Game[];
  nextCursor: string | null;
  prevCursor: string | null;
}

export interface Game {
  id: string;
  homeTeamId: string;
  homeTeam: Team;
  awayTeamId: string;
  awayTeam: Team;
  location: string;
  dateTime: string;
  cancelled: boolean;
  homeScore: number;
  awayScore: number;
  version: number;
  updatedAt: string;
  createdAt: string;
  /// Server-computed: 'scheduled' | 'live' | 'finished' | 'cancelled'
  status: GameStatus;
  events?: GameEvent[];
}

/// Initial snapshot frame delivered over the `/api/games/{id}/ws`
/// WebSocket, and the shape of the state the live-match page keeps in
/// memory.
export interface LivePoll {
  id: string;
  status: GameStatus;
  homeScore: number;
  awayScore: number;
  version: number;
  updatedAt: string;
  events: GameEvent[];
}

export interface AdjustScoreRequest {
  side: ScoreSide;
  delta: 1 | -1;
}

export interface PushSubscriptionRecord {
  id: string;
  endpoint: string;
  p256dh: string;
  auth: string;
  notifyGoal: boolean;
  createdAt: string;
}

export interface SubscribeRequest {
  endpoint: string;
  p256dh: string;
  auth: string;
  notifyGoal?: boolean;
}

export interface CreateGameRequest {
  homeTeamId: string;
  awayTeamId: string;
  location: string;
  dateTime: Date;
}

export interface UpdateGameRequest {
  homeTeamId?: string;
  awayTeamId?: string;
  location?: string;
  dateTime?: string;
  cancelled?: boolean;
  homeScore?: number;
  awayScore?: number;
}

export type EventType =
  | { type: "goal" }
  | { type: "own_goal" }
  | { type: "assist"; goalEventId: string }
  | { type: "yellow_card" }
  | { type: "red_card" };

export interface GameEvent {
  id: string;
  gameId: string;
  playerId: string | null;
  player?: Player;
  eventType: EventType;
  minute: number;
}

export interface CreateGameEventRequest {
  playerId: string | null;
  /** Required when playerId is null; identifies the scoring team. */
  teamId?: string;
  eventType: EventType;
  minute: number;
}

export interface TeamSummary {
  id: string;
  name: string;
}

export interface GameTimelineEntry {
  gameId: string;
  homeTeam: TeamSummary;
  awayTeam: TeamSummary;
  dateTime: string;
  goals: number;
  assists: number;
  yellowCards: number;
  redCards: number;
  cumulativeGoals: number;
  cumulativeAssists: number;
}

export interface PlayerGoalMatch {
  gameId: string;
  homeTeam: TeamSummary;
  awayTeam: TeamSummary;
  dateTime: string;
  homeScore: number;
  awayScore: number;
  minutes: number[];
}

export interface PlayerStats {
  playerId: string;
  appearances: number;
  goals: number;
  assists: number;
  yellowCards: number;
  redCards: number;
  goalMatches: PlayerGoalMatch[];
  gameTimeline: GameTimelineEntry[];
}

export interface LeaderboardEntry {
  playerId: string;
  firstName: string;
  lastName: string;
  shirtNumber: number;
  appearances: number;
  goals: number;
  assists: number;
  yellowCards: number;
  redCards: number;
  totalCards: number;
}

export interface Leaderboard {
  topScorers: LeaderboardEntry[];
  topAssisters: LeaderboardEntry[];
  mostCarded: LeaderboardEntry[];
}

export type Formation =
  | "4-4-2"
  | "4-3-3"
  | "4-2-3-1"
  | "3-5-2"
  | "5-3-2"
  | "4-1-4-1";

export interface LineupPlayer {
  id: string;
  firstName: string;
  lastName: string;
  shirtNumber: number;
  avatarUrl: string | null;
}

export interface LineupSlot {
  slot: number;
  captain: boolean;
  player: LineupPlayer;
}

export interface GameLineup {
  id: string;
  gameId: string;
  teamId: string;
  formation: Formation;
  updatedAt: string;
  slots: LineupSlot[];
}

export interface SaveLineupSlot {
  slot: number;
  playerId: string;
  captain: boolean;
}

export interface SaveLineupRequest {
  formation: Formation;
  slots: SaveLineupSlot[];
  teamId: string;
}

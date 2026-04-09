export type Role = 'admin' | 'moderator' | 'player';

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
    | 'Keeper'
    | 'Centrale verdediger'
    | 'Linksback'
    | 'Rechtsback'
    | 'Defensieve middenvelder'
    | 'Centrale middenvelder'
    | 'Aanvallende middenvelder'
    | 'Linksvleugel'
    | 'Rechtsvleugel'
    | 'Spits';

export interface Player {
    id: string;
    userId: string | null;
    firstName: string;
    lastName: string;
    shirtNumber: number;
    position: Position;
    active: boolean;
    user?: User | null;
}

export interface CreatePlayerRequest {
    firstName: string;
    lastName: string;
    shirtNumber: number;
    position: Position;
}

export interface UpdatePlayerRequest {
    firstName?: string;
    lastName?: string;
    shirtNumber?: number;
    position?: Position;
    active?: boolean;
}

/// Server-derived game status. Computed from `dateTime` + a fixed match
/// window on the backend; the frontend never derives liveness itself.
export type GameStatus = 'scheduled' | 'live' | 'finished' | 'cancelled';
export type HomeAway = 'home' | 'away';

export interface Game {
    id: string;
    opponent: string;
    location: string;
    dateTime: string;
    homeAway: HomeAway;
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

/// Live-poll body returned by `GET /api/games/{id}/live`.
export interface LivePoll {
    id: string;
    status: GameStatus;
    homeScore: number;
    awayScore: number;
    version: number;
    updatedAt: string;
    events: GameEvent[];
}

export type ScoreSide = 'home' | 'away';

export interface AdjustOpponentScoreRequest {
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
    opponent: string;
    location: string;
    dateTime: Date;
    homeAway: HomeAway;
}

export interface UpdateGameRequest {
    opponent?: string;
    location?: string;
    dateTime?: string;
    homeAway?: HomeAway;
    cancelled?: boolean;
    homeScore?: number;
    awayScore?: number;
}

export type EventType = 'goal' | 'assist' | 'yellow_card' | 'red_card';

export interface Player {
    id: string,
    first_name: string,
    last_name: string,
    shirt_number: number,
    position: Position,
    active: boolean,
    created_at: Date
}



export interface GameEvent {
    id: string;
    gameId: string;
    playerId: string;
    player: Player,
    eventType: EventType;
    minute: number;
}

export interface CreateGameEventRequest {
    playerId: string;
    eventType: EventType;
    minute: number;
}

export interface GameTimelineEntry {
    gameId: string;
    opponent: string;
    dateTime: string;
    goals: number;
    assists: number;
    yellowCards: number;
    redCards: number;
    cumulativeGoals: number;
    cumulativeAssists: number;
}

export interface PlayerStats {
    playerId: string;
    appearances: number;
    goals: number;
    assists: number;
    yellowCards: number;
    redCards: number;
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

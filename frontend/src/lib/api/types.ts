export type Role = 'admin' | 'moderator' | 'player';

export interface User {
    id: string;
    email: string;
    firstName: string;
    lastName: string;
    avatarUrl: string | null;
    playerId: string | null;
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

export type GameStatus = 'scheduled' | 'playing' | 'completed' | 'cancelled';
export type HomeAway = 'home' | 'away';

export interface Game {
    id: string;
    opponent: string;
    location: string;
    dateTime: string;
    homeAway: HomeAway;
    cancelled: boolean;
    homeScore: number | null;
    awayScore: number | null;
    createdAt: string;
    events?: GameEvent[];
}

export function getGameStatus(game: Game): GameStatus {
    if (game.cancelled) return 'cancelled';
    const now = Date.now();
    const start = new Date(game.dateTime).getTime();
    const end = start + 90 * 60 * 1000;
    if (now < start) return 'scheduled';
    if (now < end) return 'playing';
    return 'completed';
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
    homeScore?: number | null;
    awayScore?: number | null;
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

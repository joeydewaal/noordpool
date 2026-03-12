export type Role = 'admin' | 'moderator' | 'player';

export interface User {
    id: string;
    email: string;
    name: string;
    avatarUrl: string | null;
    roles: Role[];
}

export interface LoginRequest {
    email: string;
    password: string;
}

export interface RegisterRequest {
    name: string;
    email: string;
    password: string;
}

export interface AuthResponse {
    user: User;
    token: string;
}

export type Position = 'goalkeeper' | 'defender' | 'midfielder' | 'forward';

export interface Player {
    id: string;
    userId: string | null;
    name: string;
    shirtNumber: number;
    position: Position;
    active: boolean;
}

export interface CreatePlayerRequest {
    name: string;
    shirtNumber: number;
    position: Position;
}

export interface UpdatePlayerRequest {
    name?: string;
    shirtNumber?: number;
    position?: Position;
    active?: boolean;
}

export type GameStatus = 'scheduled' | 'completed' | 'cancelled';
export type HomeAway = 'home' | 'away';

export interface Game {
    id: string;
    opponent: string;
    location: string;
    dateTime: string;
    homeAway: HomeAway;
    status: GameStatus;
    homeScore: number | null;
    awayScore: number | null;
    createdAt: string;
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
    status?: GameStatus;
    homeScore?: number | null;
    awayScore?: number | null;
}

export type EventType = 'goal' | 'assist' | 'yellow_card' | 'red_card';

export interface GameEvent {
    id: string;
    gameId: string;
    playerId: string;
    eventType: EventType;
    minute: number;
}

export interface CreateGameEventRequest {
    playerId: string;
    eventType: EventType;
    minute: number;
}

export interface PlayerStats {
    playerId: string;
    appearances: number;
    goals: number;
    assists: number;
    yellowCards: number;
    redCards: number;
}

export interface LeaderboardEntry {
    playerId: string;
    name: string;
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

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

export type MatchStatus = 'scheduled' | 'completed' | 'cancelled';
export type HomeAway = 'home' | 'away';

export interface Match {
	id: string;
	opponent: string;
	location: string;
	dateTime: string;
	homeAway: HomeAway;
	status: MatchStatus;
	homeScore: number | null;
	awayScore: number | null;
	createdAt: string;
}

export interface CreateMatchRequest {
	opponent: string;
	location: string;
	dateTime: string;
	homeAway: HomeAway;
}

export interface UpdateMatchRequest {
	opponent?: string;
	location?: string;
	dateTime?: string;
	homeAway?: HomeAway;
	status?: MatchStatus;
	homeScore?: number | null;
	awayScore?: number | null;
}

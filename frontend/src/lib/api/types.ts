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

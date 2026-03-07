import type { AuthResponse, LoginRequest, RegisterRequest, User } from './types.js';

const MOCK_USER: User = {
	id: '1',
	email: 'player@noordpool.nl',
	name: 'Test Speler',
	avatarUrl: null,
	roles: ['admin', 'player']
};

const TOKEN_KEY = 'noordpool_token';

export async function login(data: LoginRequest): Promise<AuthResponse> {
	// Mock: accept any credentials
	const response: AuthResponse = {
		user: { ...MOCK_USER, email: data.email },
		token: 'mock-jwt-token'
	};
	localStorage.setItem(TOKEN_KEY, response.token);
	return response;
}

export async function register(data: RegisterRequest): Promise<AuthResponse> {
	const response: AuthResponse = {
		user: { ...MOCK_USER, email: data.email, name: data.name },
		token: 'mock-jwt-token'
	};
	localStorage.setItem(TOKEN_KEY, response.token);
	return response;
}

export async function me(): Promise<User | null> {
	const token = localStorage.getItem(TOKEN_KEY);
	if (!token) return null;
	return { ...MOCK_USER };
}

export function logout(): void {
	localStorage.removeItem(TOKEN_KEY);
}

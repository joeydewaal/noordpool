import type { AuthResponse, LoginRequest, RegisterRequest, User } from './types.js';
import { fetchApi, setToken, getToken, removeToken, ApiError } from './client.js';

export async function login(data: LoginRequest): Promise<AuthResponse> {
	const response = await fetchApi<AuthResponse>('/auth/login', {
		method: 'POST',
		body: JSON.stringify(data)
	});
	setToken(response.token);
	return response;
}

export async function register(data: RegisterRequest): Promise<AuthResponse> {
	const response = await fetchApi<AuthResponse>('/auth/register', {
		method: 'POST',
		body: JSON.stringify(data)
	});
	setToken(response.token);
	return response;
}

export async function me(): Promise<User | null> {
	if (!getToken()) return null;
	try {
		return await fetchApi<User>('/auth/me');
	} catch (e) {
		if (e instanceof ApiError && e.status === 401) return null;
		throw e;
	}
}

export function logout(): void {
	removeToken();
}

import type { AuthResponse, LoginRequest, RegisterRequest, User } from './types';
import { api, setToken, getToken, removeToken } from './client';

export async function login(data: LoginRequest): Promise<AuthResponse> {
	const response = await api.post<AuthResponse>('/auth/login', data);
	setToken(response.data.token);
	return response.data;
}

export async function register(data: RegisterRequest): Promise<AuthResponse> {
	const response = await api.post<AuthResponse>('/auth/register', data);
	setToken(response.data.token);
	return response.data;
}

export async function me(): Promise<User | null> {
	if (!getToken()) return null;
	try {
		const response = await api.get<User>('/auth/me');
		return response.data;
	} catch (e: any) {
		if (e.response?.status === 401) return null;
		throw e;
	}
}

export function logout(): void {
	removeToken();
}

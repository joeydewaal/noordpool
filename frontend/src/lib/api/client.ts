import axios from 'axios';

const TOKEN_KEY = 'noordpool_token';

export function getToken(): string | null {
	return localStorage.getItem(TOKEN_KEY);
}

export function setToken(token: string): void {
	localStorage.setItem(TOKEN_KEY, token);
}

export function removeToken(): void {
	localStorage.removeItem(TOKEN_KEY);
}

export const api = axios.create({ baseURL: '/api' });

api.interceptors.request.use((config) => {
	const token = getToken();
	if (token) {
		config.headers.Authorization = `Bearer ${token}`;
	}
	return config;
});

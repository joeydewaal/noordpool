const TOKEN_KEY = 'noordpool_token';

export class ApiError extends Error {
	constructor(
		public status: number,
		public body: string
	) {
		super(`API error ${status}: ${body}`);
	}
}

export function getToken(): string | null {
	return localStorage.getItem(TOKEN_KEY);
}

export function setToken(token: string): void {
	localStorage.setItem(TOKEN_KEY, token);
}

export function removeToken(): void {
	localStorage.removeItem(TOKEN_KEY);
}

export async function fetchApi<T>(path: string, options: RequestInit = {}, customFetch: typeof fetch = fetch): Promise<T> {
	const token = getToken();
	const headers: Record<string, string> = {};

	if (token) {
		headers['Authorization'] = `Bearer ${token}`;
	}

	const method = (options.method ?? 'GET').toUpperCase();
	if (method === 'POST' || method === 'PUT') {
		headers['Content-Type'] = 'application/json';
	}

	const res = await customFetch(`/api${path}`, {
		...options,
		headers: { ...headers, ...options.headers }
	});

	if (!res.ok) {
		const body = await res.text();
		throw new ApiError(res.status, body);
	}

	if (res.status === 204) return undefined as T;
	return res.json();
}

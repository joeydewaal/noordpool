import type { Match, CreateMatchRequest, UpdateMatchRequest } from './types.js';
import { fetchApi } from './client.js';

export async function getMatches(): Promise<Match[]> {
	return fetchApi<Match[]>('/matches');
}

export async function getMatch(id: string): Promise<Match | null> {
	try {
		return await fetchApi<Match>(`/matches/${id}`);
	} catch {
		return null;
	}
}

export async function getUpcomingMatches(limit?: number): Promise<Match[]> {
	const params = limit ? `?limit=${limit}` : '';
	return fetchApi<Match[]>(`/matches/upcoming${params}`);
}

export async function getRecentResults(limit?: number): Promise<Match[]> {
	const params = limit ? `?limit=${limit}` : '';
	return fetchApi<Match[]>(`/matches/recent${params}`);
}

export async function createMatch(data: CreateMatchRequest): Promise<Match> {
	return fetchApi<Match>('/matches', {
		method: 'POST',
		body: JSON.stringify(data)
	});
}

export async function updateMatch(id: string, data: UpdateMatchRequest): Promise<Match> {
	return fetchApi<Match>(`/matches/${id}`, {
		method: 'PUT',
		body: JSON.stringify(data)
	});
}

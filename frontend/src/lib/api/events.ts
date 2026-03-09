import type { MatchEvent, CreateMatchEventRequest, PlayerStats, Leaderboard } from './types.js';
import { fetchApi } from './client.js';

export async function getMatchEvents(matchId: string): Promise<MatchEvent[]> {
	return fetchApi<MatchEvent[]>(`/matches/${matchId}/events`);
}

export async function createMatchEvent(matchId: string, data: CreateMatchEventRequest): Promise<MatchEvent> {
	return fetchApi<MatchEvent>(`/matches/${matchId}/events`, {
		method: 'POST',
		body: JSON.stringify(data)
	});
}

export async function deleteMatchEvent(matchId: string, eventId: string): Promise<void> {
	return fetchApi<void>(`/matches/${matchId}/events/${eventId}`, {
		method: 'DELETE'
	});
}

export async function getPlayerStats(playerId: string): Promise<PlayerStats> {
	return fetchApi<PlayerStats>(`/players/${playerId}/stats`);
}

export async function getLeaderboard(): Promise<Leaderboard> {
	return fetchApi<Leaderboard>('/stats/leaderboard');
}

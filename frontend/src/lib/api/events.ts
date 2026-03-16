import type { GameEvent, CreateGameEventRequest, PlayerStats, Leaderboard } from './types.ts';
import { fetchApi } from './client.ts';

export async function getGameEvents(gameId: string): Promise<GameEvent[]> {
	return fetchApi<GameEvent[]>(`/games/${gameId}/events`);
}

export async function createGameEvent(gameId: string, data: CreateGameEventRequest): Promise<GameEvent> {
	return fetchApi<GameEvent>(`/games/${gameId}/events`, {
		method: 'POST',
		body: JSON.stringify(data)
	});
}

export async function deleteGameEvent(gameId: string, eventId: string): Promise<void> {
	return fetchApi<void>(`/games/${gameId}/events/${eventId}`, {
		method: 'DELETE'
	});
}

export async function getPlayerStats(playerId: string): Promise<PlayerStats> {
	return fetchApi<PlayerStats>(`/players/${playerId}/stats`);
}

export async function getLeaderboard(): Promise<Leaderboard> {
	return fetchApi<Leaderboard>('/stats/leaderboard');
}

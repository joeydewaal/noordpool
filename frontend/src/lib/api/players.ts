import type { Player, CreatePlayerRequest, UpdatePlayerRequest } from './types.ts';
import { fetchApi } from './client.ts';

export async function getPlayers(): Promise<Player[]> {
	return fetchApi<Player[]>('/players');
}

export async function getPlayer(id: string): Promise<Player | null> {
	try {
		return await fetchApi<Player>(`/players/${id}`);
	} catch {
		return null;
	}
}

export async function createPlayer(data: CreatePlayerRequest): Promise<Player> {
	return fetchApi<Player>('/players', {
		method: 'POST',
		body: JSON.stringify(data)
	});
}

export async function updatePlayer(id: string, data: UpdatePlayerRequest): Promise<Player> {
	return fetchApi<Player>(`/players/${id}`, {
		method: 'PUT',
		body: JSON.stringify(data)
	});
}

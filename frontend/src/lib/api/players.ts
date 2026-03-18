import type { Player, CreatePlayerRequest, UpdatePlayerRequest } from './types';
import { api } from './client';

export async function getPlayers(): Promise<Player[]> {
	return (await api.get<Player[]>('/players')).data;
}

export async function getPlayer(id: string): Promise<Player | null> {
	try {
		return (await api.get<Player>(`/players/${id}`)).data;
	} catch {
		return null;
	}
}

export async function createPlayer(data: CreatePlayerRequest): Promise<Player> {
	return (await api.post<Player>('/players', data)).data;
}

export async function updatePlayer(id: string, data: UpdatePlayerRequest): Promise<Player> {
	return (await api.put<Player>(`/players/${id}`, data)).data;
}

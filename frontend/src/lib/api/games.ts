import type { Game, CreateGameRequest, UpdateGameRequest } from './types.ts';
import { fetchApi } from './client.ts';

export async function getGames(): Promise<Game[]> {
    return fetchApi<Game[]>('/games');
}

export async function getGame(id: string): Promise<Game | null> {
    try {
        return await fetchApi<Game>(`/games/${id}`);
    } catch {
        return null;
    }
}

export async function getUpcomingGames(limit?: number): Promise<Game[]> {
    const params = limit ? `?limit=${limit}` : '';
    return fetchApi<Game[]>(`/games/upcoming${params}`);
}

export async function getRecentResults(limit?: number): Promise<Game[]> {
    const params = limit ? `?limit=${limit}` : '';
    return fetchApi<Game[]>(`/games/recent${params}`);
}

export async function createGame(data: CreateGameRequest): Promise<Game> {
    return fetchApi<Game>('/games', {
        method: 'POST',
        body: JSON.stringify(data)
    });
}

export async function updateGame(id: string, data: UpdateGameRequest): Promise<Game> {
    return fetchApi<Game>(`/games/${id}`, {
        method: 'PUT',
        body: JSON.stringify(data)
    });
}

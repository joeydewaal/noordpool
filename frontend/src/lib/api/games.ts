import type { Game, CreateGameRequest, UpdateGameRequest } from './types';
import { api } from './client';

export async function getGames(): Promise<Game[]> {
    return (await api.get<Game[]>('/games')).data;
}

export async function getGame(id: string): Promise<Game | null> {
    try {
        return (await api.get<Game>(`/games/${id}`)).data;
    } catch {
        return null;
    }
}

export async function getUpcomingGames(limit?: number): Promise<Game[]> {
    const params = limit ? { limit } : {};
    return (await api.get<Game[]>('/games/upcoming', { params })).data;
}

export async function getRecentResults(limit?: number): Promise<Game[]> {
    const params = limit ? { limit } : {};
    return (await api.get<Game[]>('/games/recent', { params })).data;
}

export async function createGame(data: CreateGameRequest): Promise<Game> {
    return (await api.post<Game>('/games', data)).data;
}

export async function updateGame(id: string, data: UpdateGameRequest): Promise<Game> {
    return (await api.put<Game>(`/games/${id}`, data)).data;
}

import type {
    Game,
    CreateGameRequest,
    UpdateGameRequest,
    LivePoll,
    AdjustScoreRequest,
} from './types';
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

export async function getGamesSummary(limit = 3): Promise<{ upcoming: Game[]; recent: Game[] }> {
    return (await api.get('/games/summary', { params: { limit } })).data;
}

export async function createGame(data: CreateGameRequest): Promise<Game> {
    return (await api.post<Game>('/games', data)).data;
}

export async function updateGame(id: string, data: UpdateGameRequest): Promise<Game> {
    return (await api.put<Game>(`/games/${id}`, data)).data;
}

/// Result of a live-poll request. `null` body means 304 Not Modified —
/// the caller should keep its previous state. Always returns the latest
/// `etag` so the caller can pass it back on the next request.
export interface LivePollResult {
    body: LivePoll | null;
    etag: string | null;
}

export async function pollLive(id: string, etag: string | null): Promise<LivePollResult> {
    const headers: Record<string, string> = {};
    if (etag) headers['If-None-Match'] = etag;

    const res = await api.get<LivePoll>(`/games/${id}/live`, {
        headers,
        // Treat 304 as a successful response so axios doesn't throw.
        validateStatus: (s) => s === 200 || s === 304,
    });

    if (res.status === 304) {
        return { body: null, etag };
    }
    return {
        body: res.data,
        etag: (res.headers['etag'] as string | undefined) ?? null,
    };
}

export async function adjustLiveScore(id: string, data: AdjustScoreRequest): Promise<LivePoll> {
    return (await api.post<LivePoll>(`/games/${id}/live/score`, data)).data;
}

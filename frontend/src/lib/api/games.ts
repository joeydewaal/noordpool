import type {
  Game,
  GamesPage,
  CreateGameRequest,
  UpdateGameRequest,
  LivePoll,
  AdjustScoreRequest,
  ScoreSide,
} from "./types";
import { api } from "./client";
export type { LivePoll };

export async function getGames(opts?: {
  limit?: number;
  after?: string;
}): Promise<GamesPage> {
  const params: Record<string, string | number> = {};
  if (opts?.limit) params.limit = opts.limit;
  if (opts?.after) params.after = opts.after;
  return (await api.get<GamesPage>("/games", { params })).data;
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
  return (await api.get<Game[]>("/games/upcoming", { params })).data;
}

export async function getRecentResults(limit?: number): Promise<Game[]> {
  const params = limit ? { limit } : {};
  return (await api.get<Game[]>("/games/recent", { params })).data;
}

export async function getGamesSummary(
  limit = 3,
): Promise<{ upcoming: Game[]; recent: Game[] }> {
  return (await api.get("/games/summary", { params: { limit } })).data;
}

export async function createGame(data: CreateGameRequest): Promise<Game> {
  return (await api.post<Game>("/games", data)).data;
}

export async function updateGame(
  id: string,
  data: UpdateGameRequest,
): Promise<Game> {
  return (await api.put<Game>(`/games/${id}`, data)).data;
}

export async function deleteGame(id: string): Promise<void> {
  await api.delete(`/games/${id}`);
}

/// Adjusts the live score for the specified side by +/-1.
export async function adjustScore(
  id: string,
  side: ScoreSide,
  delta: 1 | -1,
): Promise<LivePoll> {
  const body: AdjustScoreRequest = { side, delta };
  return (await api.post<LivePoll>(`/games/${id}/live/score`, body)).data;
}

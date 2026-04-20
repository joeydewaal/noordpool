import type { GameLineup, SaveLineupRequest } from "./types";
import { api } from "./client";

export async function getLineup(gameId: string): Promise<GameLineup | null> {
  try {
    return (await api.get<GameLineup>(`/games/${gameId}/lineup`)).data;
  } catch {
    return null;
  }
}

export async function saveLineup(
  gameId: string,
  data: SaveLineupRequest,
): Promise<GameLineup> {
  return (await api.put<GameLineup>(`/games/${gameId}/lineup`, data)).data;
}

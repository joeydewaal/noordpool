import type { GameLineup, SaveLineupRequest } from "./types";
import { api } from "./client";

export async function getLineup(
  gameId: string,
  teamId?: string,
): Promise<GameLineup | null> {
  try {
    const qs = teamId ? `?teamId=${teamId}` : "";
    return (await api.get<GameLineup>(`/games/${gameId}/lineup${qs}`)).data;
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

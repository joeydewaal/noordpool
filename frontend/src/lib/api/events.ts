import type {
  GameEvent,
  CreateGameEventRequest,
  PlayerStats,
  Leaderboard,
} from "./types";
import { api } from "./client";

export async function getGameEvents(gameId: string): Promise<GameEvent[]> {
  return (await api.get<GameEvent[]>(`/games/${gameId}/events`)).data;
}

export async function createGameEvent(
  gameId: string,
  data: CreateGameEventRequest,
): Promise<GameEvent> {
  return (await api.post<GameEvent>(`/games/${gameId}/events`, data)).data;
}

export async function deleteGameEvent(
  gameId: string,
  eventId: string,
): Promise<void> {
  await api.delete(`/games/${gameId}/events/${eventId}`);
}

export async function getPlayerStats(playerId: string): Promise<PlayerStats> {
  return (await api.get<PlayerStats>(`/players/${playerId}/stats`)).data;
}

export async function getLeaderboard(): Promise<Leaderboard> {
  return (await api.get<Leaderboard>("/stats/leaderboard")).data;
}

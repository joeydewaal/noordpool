import type { Team } from "./types";
import { api } from "./client";

export async function listTeams(): Promise<Team[]> {
  return (await api.get<Team[]>("/teams")).data;
}

export async function createTeam(name: string): Promise<Team> {
  return (await api.post<Team>("/teams", { name })).data;
}

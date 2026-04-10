import type {
  AuthResponse,
  LoginRequest,
  PlayerMatch,
  RegisterRequest,
  User,
} from "./types";
import { api, setToken, getToken, removeToken } from "./client";

export async function login(data: LoginRequest): Promise<AuthResponse> {
  const response = await api.post<AuthResponse>("/auth/login", data);
  setToken(response.data.token);
  return response.data;
}

export async function register(data: RegisterRequest): Promise<AuthResponse> {
  const response = await api.post<AuthResponse>("/auth/register", data);
  setToken(response.data.token);
  return response.data;
}

export async function linkPlayer(playerId: string): Promise<AuthResponse> {
  const response = await api.post<AuthResponse>("/auth/link-player", {
    player_id: playerId,
  });
  setToken(response.data.token);
  return response.data;
}

export async function unlinkPlayer(): Promise<AuthResponse> {
  const response = await api.post<AuthResponse>("/auth/unlink-player");
  setToken(response.data.token);
  return response.data;
}

export async function me(): Promise<User | null> {
  if (!getToken()) return null;
  try {
    const response = await api.get<User>("/auth/me");
    return response.data;
  } catch (e: any) {
    if (e.response?.status === 401) return null;
    throw e;
  }
}

export async function findPlayer(name: string): Promise<PlayerMatch[]> {
  const response = await api.get<PlayerMatch[]>("/auth/find-player", {
    params: { name },
  });
  return response.data;
}

export function logout(): void {
  removeToken();
}

import type { User } from "$lib/api/types";
import { getToken } from "$lib/api/client";

function teamIdFromToken(): string | null {
  const token = getToken();
  if (!token) return null;
  try {
    const payload = JSON.parse(atob(token.split(".")[1]));
    return payload.team_id ?? null;
  } catch {
    return null;
  }
}

class AuthState {
  user: User | null = $state(null);
  loading: boolean = $state(true);
  teamId: string | null = $state(null);

  get isAuthenticated() {
    return this.user !== null;
  }

  get isAdmin() {
    return this.user?.roles.includes("admin") ?? false;
  }

  get isModerator() {
    return this.user?.roles.includes("moderator") ?? false;
  }

  get playerId() {
    return this.user?.playerId ?? null;
  }

  setUser(user: User) {
    this.user = user;
    this.teamId = teamIdFromToken();
  }

  clear() {
    this.user = null;
    this.teamId = null;
  }
}

export const auth = new AuthState();

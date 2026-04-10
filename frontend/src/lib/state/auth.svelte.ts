import type { User } from "$lib/api/types";

class AuthState {
  user: User | null = $state(null);
  loading: boolean = $state(true);

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
  }

  clear() {
    this.user = null;
  }
}

export const auth = new AuthState();

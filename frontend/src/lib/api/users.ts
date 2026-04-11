import type { User } from "./types";
import { api } from "./client";

export interface UpdateUserRequest {
  isModerator?: boolean;
}

export async function listUsers(): Promise<User[]> {
  return (await api.get<User[]>("/users")).data;
}

export async function updateUser(
  id: string,
  data: UpdateUserRequest,
): Promise<User> {
  return (await api.patch<User>(`/users/${id}`, data)).data;
}

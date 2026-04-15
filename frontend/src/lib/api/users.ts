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

export async function uploadAvatar(file: File): Promise<User> {
  const form = new FormData();
  form.append("file", file);
  return (
    await api.post<User>("/users/me/avatar", form, {
      headers: { "Content-Type": "multipart/form-data" },
    })
  ).data;
}

export async function deleteAvatar(): Promise<User> {
  return (await api.delete<User>("/users/me/avatar")).data;
}

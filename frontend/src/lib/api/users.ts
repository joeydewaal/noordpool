import type { User } from "./types";
import { api } from "./client";

export interface UpdateUserRequest {
  isModerator?: boolean;
}

interface PresignResponse {
  uploadUrl: string;
  publicUrl: string;
  contentType: string;
  key: string;
}

const AVATAR_SIZE = 256;

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
  const blob = await resizeToWebp(file, AVATAR_SIZE);
  const presign = (await api.post<PresignResponse>("/users/me/avatar/presign"))
    .data;
  const put = await fetch(presign.uploadUrl, {
    method: "PUT",
    headers: { "Content-Type": presign.contentType },
    body: blob,
  });
  if (!put.ok) {
    throw new Error(`Avatar upload failed: HTTP ${put.status}`);
  }
  return (await api.post<User>("/users/me/avatar", { url: presign.publicUrl }))
    .data;
}

export async function deleteAvatar(): Promise<User> {
  return (await api.delete<User>("/users/me/avatar")).data;
}

async function resizeToWebp(file: File, size: number): Promise<Blob> {
  const bitmap = await createImageBitmap(file);
  const side = Math.min(bitmap.width, bitmap.height);
  const sx = Math.floor((bitmap.width - side) / 2);
  const sy = Math.floor((bitmap.height - side) / 2);

  const canvas = document.createElement("canvas");
  canvas.width = size;
  canvas.height = size;
  const ctx = canvas.getContext("2d");
  if (!ctx) throw new Error("canvas 2D context unavailable");
  ctx.drawImage(bitmap, sx, sy, side, side, 0, 0, size, size);
  bitmap.close?.();

  return await new Promise<Blob>((resolve, reject) => {
    canvas.toBlob(
      (blob) => {
        if (blob) resolve(blob);
        else reject(new Error("canvas.toBlob returned null"));
      },
      "image/webp",
      0.9,
    );
  });
}

/// <reference lib="webworker" />
/// <reference types="@sveltejs/kit" />

// Custom service worker (injectManifest mode). Handles:
//   - Workbox precache + runtime caching for /api/{games,players,stats}
//   - Web Push (`push`, `notificationclick`) for live goal alerts
//
// We use injectManifest (instead of generateSW) so we can write the push
// handler ourselves — Workbox doesn't generate one.

import { precacheAndRoute } from "workbox-precaching";
import { registerRoute } from "workbox-routing";
import { StaleWhileRevalidate } from "workbox-strategies";
import { ExpirationPlugin } from "workbox-expiration";
import { clientsClaim } from "workbox-core";

declare let self: ServiceWorkerGlobalScope;

// Take over immediately on update so a new SW (e.g. one that gained the
// `push` handler) doesn't get stuck in `waiting` while an old SW keeps
// silently dropping push events. Without this, an installed PWA can sit on
// a stale SW indefinitely because the window never fully closes.
self.skipWaiting();
clientsClaim();

// Workbox precache manifest is injected here at build time.
precacheAndRoute(self.__WB_MANIFEST);

registerRoute(
  ({ url }) => /\/api\/(games|players|stats)/.test(url.pathname),
  new StaleWhileRevalidate({
    cacheName: "api-cache",
    plugins: [
      new ExpirationPlugin({
        maxEntries: 50,
        maxAgeSeconds: 60 * 60 * 24,
      }),
    ],
  }),
);

// ---------- Web Push ----------
//
// Payload contract (set in `backend/src/push/mod.rs::notify_goal`):
//   {
//     type: "goal",
//     gameId: string,
//     homeTeam: { id: string, name: string },
//     awayTeam: { id: string, name: string },
//     homeScore: number,
//     awayScore: number,
//     side: "home" | "away",
//   }

interface GoalPayload {
  type: "goal";
  gameId: string;
  homeTeam: { id: string; name: string };
  awayTeam: { id: string; name: string };
  homeScore: number;
  awayScore: number;
  side: "home" | "away";
}

self.addEventListener("push", (event: PushEvent) => {
  let payload: GoalPayload | null = null;
  try {
    payload = event.data?.json() as GoalPayload;
  } catch {
    // Ignore malformed payloads — show a generic notification.
  }

  const title =
    payload?.type === "goal"
      ? `Goal! ${payload.homeScore}–${payload.awayScore}`
      : "Noordpool";

  const body =
    payload?.type === "goal"
      ? `${payload.homeTeam.name} ${payload.homeScore}–${payload.awayScore} ${payload.awayTeam.name}`
      : "Update beschikbaar";

  const data =
    payload?.type === "goal"
      ? { url: `/games/${payload.gameId}` }
      : { url: "/" };

  event.waitUntil(
    self.registration.showNotification(title, {
      body,
      icon: "/icons/icon-192.png",
      badge: "/icons/icon-192.png",
      data,
      tag: payload?.type === "goal" ? `goal-${payload.gameId}` : "noordpool",
    }),
  );
});

self.addEventListener("notificationclick", (event: NotificationEvent) => {
  event.notification.close();
  const url =
    (event.notification.data as { url?: string } | undefined)?.url ?? "/";

  event.waitUntil(
    (async () => {
      const allClients = await self.clients.matchAll({
        type: "window",
        includeUncontrolled: true,
      });
      for (const client of allClients) {
        // If the target page is already open, just focus it.
        if (client.url.endsWith(url) && "focus" in client) {
          return client.focus();
        }
      }
      if (self.clients.openWindow) {
        return self.clients.openWindow(url);
      }
    })(),
  );
});

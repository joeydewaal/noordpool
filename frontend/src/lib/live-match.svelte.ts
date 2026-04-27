import type { GameEvent, GameStatus, LivePoll } from "./api/types";

/**
 * Live-match WebSocket driver.
 *
 * Connects to `/api/games/:id/ws` and dispatches typed events to the caller.
 * The socket sends a `snapshot` frame on connect, then incremental updates
 * (`scoreUpdate`, `eventAdded`, `eventDeleted`, `statusChange`).
 *
 * The driver reconnects automatically with exponential back-off and
 * disconnects after 60s of the tab being hidden to avoid holding idle
 * connections; it reconnects on refocus.
 */

interface ScoreUpdateFrame {
  type: "scoreUpdate";
  home: number;
  away: number;
  version: number;
  updatedAt: string;
}

interface SnapshotFrame {
  type: "snapshot";
  id: string;
  status: GameStatus;
  homeScore: number;
  awayScore: number;
  version: number;
  updatedAt: string;
  events: GameEvent[];
}

interface EventAddedFrame {
  type: "eventAdded";
  // server emits the full GameEvent flattened in the tagged union
  [key: string]: unknown;
}

interface EventDeletedFrame {
  type: "eventDeleted";
  id: string;
}

interface StatusChangeFrame {
  type: "statusChange";
  status: GameStatus;
}

type Frame =
  | SnapshotFrame
  | ScoreUpdateFrame
  | EventAddedFrame
  | EventDeletedFrame
  | StatusChangeFrame;

export interface LiveMatchHandlers {
  onSnapshot?: (snapshot: LivePoll) => void;
  onScoreUpdate?: (update: {
    home: number;
    away: number;
    version: number;
    updatedAt: string;
  }) => void;
  onEventAdded?: (event: GameEvent) => void;
  onEventDeleted?: (id: string) => void;
  onStatusChange?: (status: GameStatus) => void;
}

const BACKOFF_MS = [1_000, 2_000, 4_000, 8_000, 16_000, 30_000];
const HIDDEN_GRACE_MS = 60_000;

export function startLiveMatchStream(
  gameId: string,
  handlers: LiveMatchHandlers,
): () => void {
  let socket: WebSocket | null = null;
  let stopped = false;
  let attempt = 0;
  let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  let hiddenTimer: ReturnType<typeof setTimeout> | null = null;

  function url(): string {
    const base = import.meta.env.VITE_API_BASE_URL;
    if (base) {
      const wsBase = base.replace(/^http/, "ws").replace(/\/$/, "");
      return `${wsBase}/games/${gameId}/ws`;
    }
    const proto = location.protocol === "https:" ? "wss:" : "ws:";
    return `${proto}//${location.host}/api/games/${gameId}/ws`;
  }

  function scheduleReconnect() {
    if (stopped || reconnectTimer) return;
    const delay = BACKOFF_MS[Math.min(attempt, BACKOFF_MS.length - 1)];
    attempt += 1;
    reconnectTimer = setTimeout(() => {
      reconnectTimer = null;
      connect();
    }, delay);
  }

  function connect() {
    if (stopped) return;
    try {
      socket = new WebSocket(url());
    } catch {
      scheduleReconnect();
      return;
    }

    socket.onopen = () => {
      attempt = 0;
    };

    socket.onmessage = (ev) => {
      let frame: Frame;
      try {
        frame = JSON.parse(ev.data) as Frame;
      } catch {
        return;
      }
      dispatch(frame, handlers);
    };

    socket.onclose = () => {
      socket = null;
      if (!stopped) scheduleReconnect();
    };

    socket.onerror = () => {
      // onclose will fire right after; let it handle reconnect.
    };
  }

  function disconnect() {
    if (reconnectTimer) {
      clearTimeout(reconnectTimer);
      reconnectTimer = null;
    }
    if (socket) {
      socket.onclose = null;
      socket.onerror = null;
      socket.onmessage = null;
      try {
        socket.close();
      } catch {
        // ignore
      }
      socket = null;
    }
  }

  function onVisibilityChange() {
    if (stopped) return;
    if (document.visibilityState === "hidden") {
      // Give the user 60s of grace before dropping the socket — tab
      // switches are usually brief.
      hiddenTimer = setTimeout(() => {
        hiddenTimer = null;
        disconnect();
      }, HIDDEN_GRACE_MS);
    } else {
      if (hiddenTimer) {
        clearTimeout(hiddenTimer);
        hiddenTimer = null;
      }
      if (!socket) {
        attempt = 0;
        connect();
      }
    }
  }

  connect();
  if (typeof document !== "undefined") {
    document.addEventListener("visibilitychange", onVisibilityChange);
  }

  return () => {
    stopped = true;
    if (hiddenTimer) clearTimeout(hiddenTimer);
    disconnect();
    if (typeof document !== "undefined") {
      document.removeEventListener("visibilitychange", onVisibilityChange);
    }
  };
}

function dispatch(frame: Frame, handlers: LiveMatchHandlers) {
  switch (frame.type) {
    case "snapshot": {
      const { id, status, homeScore, awayScore, version, updatedAt, events } =
        frame;
      handlers.onSnapshot?.({
        id,
        status,
        homeScore,
        awayScore,
        version,
        updatedAt,
        events,
      });
      break;
    }
    case "scoreUpdate":
      handlers.onScoreUpdate?.({
        home: frame.home,
        away: frame.away,
        version: frame.version,
        updatedAt: frame.updatedAt,
      });
      break;
    case "eventAdded": {
      const { type: _t, ...rest } = frame;
      handlers.onEventAdded?.(rest as unknown as GameEvent);
      break;
    }
    case "eventDeleted":
      handlers.onEventDeleted?.(frame.id);
      break;
    case "statusChange":
      handlers.onStatusChange?.(frame.status);
      break;
  }
}

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const apiMocks = vi.hoisted(() => ({
  getVapidPublicKey: vi.fn(),
  listMyPushSubscriptions: vi.fn(),
  subscribePush: vi.fn(),
  unsubscribePush: vi.fn(),
}));

vi.mock("./api/push", () => apiMocks);

import {
  disablePush,
  enablePush,
  isCurrentBrowserSubscribed,
  isPushSupported,
} from "./push-subscribe";

// A base64url-no-padding fake VAPID public key (raw bytes don't matter — we
// just need urlBase64ToUint8Array to accept it).
const FAKE_VAPID_KEY = "AAECAwQFBgcICQ";

function makeKey(byte: number, len = 8): ArrayBuffer {
  const buf = new ArrayBuffer(len);
  new Uint8Array(buf).fill(byte);
  return buf;
}

interface FakeSubscription {
  endpoint: string;
  getKey: (name: "p256dh" | "auth") => ArrayBuffer | null;
  unsubscribe: ReturnType<typeof vi.fn>;
}

function fakeSubscription(
  endpoint = "https://push.example/abc",
): FakeSubscription {
  return {
    endpoint,
    getKey: (name) => (name === "p256dh" ? makeKey(1, 65) : makeKey(2, 16)),
    unsubscribe: vi.fn().mockResolvedValue(true),
  };
}

interface FakeRegistration {
  pushManager: {
    getSubscription: ReturnType<typeof vi.fn>;
    subscribe: ReturnType<typeof vi.fn>;
  };
}

function installBrowserPushStubs(
  opts: {
    permission?: NotificationPermission;
    existingSub?: FakeSubscription | null;
    subscribeResult?: FakeSubscription;
  } = {},
) {
  const permission = opts.permission ?? "granted";
  const existing = opts.existingSub === undefined ? null : opts.existingSub;
  const subscribed = opts.subscribeResult ?? fakeSubscription();

  const registration: FakeRegistration = {
    pushManager: {
      getSubscription: vi.fn().mockResolvedValue(existing),
      subscribe: vi.fn().mockResolvedValue(subscribed),
    },
  };

  Object.defineProperty(globalThis, "Notification", {
    configurable: true,
    writable: true,
    value: {
      permission,
      requestPermission: vi.fn().mockResolvedValue(permission),
    },
  });

  Object.defineProperty(globalThis, "PushManager", {
    configurable: true,
    writable: true,
    value: function PushManager() {},
  });

  Object.defineProperty(navigator, "serviceWorker", {
    configurable: true,
    value: {
      ready: Promise.resolve(registration),
    },
  });

  return { registration, subscribed };
}

describe("isPushSupported", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("returns true when all browser APIs are present", () => {
    installBrowserPushStubs();
    expect(isPushSupported()).toBe(true);
  });

  it("returns false when Notification is missing", () => {
    installBrowserPushStubs();
    // @ts-expect-error — intentionally clobbering global
    delete globalThis.Notification;
    expect(isPushSupported()).toBe(false);
  });
});

describe("enablePush", () => {
  beforeEach(() => {
    Object.values(apiMocks).forEach((m) => m.mockReset());
    apiMocks.getVapidPublicKey.mockResolvedValue(FAKE_VAPID_KEY);
    apiMocks.subscribePush.mockResolvedValue(undefined);
  });

  it("subscribes via pushManager and POSTs the result to the backend", async () => {
    const { registration, subscribed } = installBrowserPushStubs();

    await enablePush();

    expect(registration.pushManager.subscribe).toHaveBeenCalledWith(
      expect.objectContaining({ userVisibleOnly: true }),
    );
    expect(apiMocks.subscribePush).toHaveBeenCalledTimes(1);
    const payload = apiMocks.subscribePush.mock.calls[0][0];
    expect(payload.endpoint).toBe(subscribed.endpoint);
    expect(payload.notifyGoal).toBe(true);
    expect(payload.p256dh).toMatch(/^[A-Za-z0-9_-]+$/); // base64url no padding
    expect(payload.auth).toMatch(/^[A-Za-z0-9_-]+$/);
  });

  it("reuses an existing browser subscription instead of re-subscribing", async () => {
    const existing = fakeSubscription("https://push.example/existing");
    const { registration } = installBrowserPushStubs({ existingSub: existing });

    await enablePush();

    expect(registration.pushManager.subscribe).not.toHaveBeenCalled();
    expect(apiMocks.subscribePush).toHaveBeenCalledTimes(1);
    expect(apiMocks.subscribePush.mock.calls[0][0].endpoint).toBe(
      "https://push.example/existing",
    );
  });

  it("throws a Dutch error when permission is denied", async () => {
    installBrowserPushStubs({ permission: "denied" });
    await expect(enablePush()).rejects.toThrow(/geweigerd/i);
    expect(apiMocks.subscribePush).not.toHaveBeenCalled();
  });
});

describe("disablePush", () => {
  beforeEach(() => {
    Object.values(apiMocks).forEach((m) => m.mockReset());
    apiMocks.unsubscribePush.mockResolvedValue(undefined);
  });

  it("removes the server record and unsubscribes the browser", async () => {
    const existing = fakeSubscription("https://push.example/xyz");
    installBrowserPushStubs({ existingSub: existing });

    await disablePush();

    expect(apiMocks.unsubscribePush).toHaveBeenCalledWith(
      "https://push.example/xyz",
    );
    expect(existing.unsubscribe).toHaveBeenCalled();
  });

  it("still calls browser unsubscribe even if the server call fails", async () => {
    const existing = fakeSubscription();
    installBrowserPushStubs({ existingSub: existing });
    apiMocks.unsubscribePush.mockRejectedValueOnce(new Error("500"));

    await expect(disablePush()).rejects.toThrow("500");
    expect(existing.unsubscribe).toHaveBeenCalled();
  });

  it("is a no-op when no browser subscription exists", async () => {
    installBrowserPushStubs({ existingSub: null });
    await disablePush();
    expect(apiMocks.unsubscribePush).not.toHaveBeenCalled();
  });
});

describe("isCurrentBrowserSubscribed", () => {
  beforeEach(() => {
    Object.values(apiMocks).forEach((m) => m.mockReset());
  });

  it("is true when the server knows about the current browser endpoint", async () => {
    const existing = fakeSubscription("https://push.example/match");
    installBrowserPushStubs({ existingSub: existing });
    apiMocks.listMyPushSubscriptions.mockResolvedValue([
      { id: "1", endpoint: "https://push.example/match", notifyGoal: true },
    ]);

    await expect(isCurrentBrowserSubscribed()).resolves.toBe(true);
  });

  it("is false when the server has pruned the row", async () => {
    installBrowserPushStubs({
      existingSub: fakeSubscription("https://push.example/stale"),
    });
    apiMocks.listMyPushSubscriptions.mockResolvedValue([]);

    await expect(isCurrentBrowserSubscribed()).resolves.toBe(false);
  });

  it("is false when the browser has no subscription", async () => {
    installBrowserPushStubs({ existingSub: null });
    await expect(isCurrentBrowserSubscribed()).resolves.toBe(false);
    expect(apiMocks.listMyPushSubscriptions).not.toHaveBeenCalled();
  });
});

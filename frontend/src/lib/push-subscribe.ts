import {
    getVapidPublicKey,
    listMyPushSubscriptions,
    subscribePush,
    unsubscribePush,
} from './api/push';

/**
 * Browser-side helpers around the Web Push subscription lifecycle.
 *
 * Lifecycle:
 *   1. The page asks the user for Notification permission.
 *   2. We fetch the server's VAPID public key (b64url) and convert it to a
 *      Uint8Array — that's the format `pushManager.subscribe` wants.
 *   3. The browser produces a `PushSubscription` (endpoint + p256dh + auth)
 *      that we POST to `/api/push/subscriptions`.
 *
 * Errors are intentionally surfaced as plain `Error` instances; the calling
 * UI translates them into Dutch user-facing messages.
 */

export function isPushSupported(): boolean {
    return (
        typeof window !== 'undefined' &&
        'serviceWorker' in navigator &&
        'PushManager' in window &&
        'Notification' in window
    );
}

/// Convert a base64url-encoded VAPID public key to the Uint8Array
/// `pushManager.subscribe` expects.
function urlBase64ToUint8Array(b64: string): Uint8Array<ArrayBuffer> {
    const padding = '='.repeat((4 - (b64.length % 4)) % 4);
    const base64 = (b64 + padding).replace(/-/g, '+').replace(/_/g, '/');
    const raw = atob(base64);
    const buffer = new ArrayBuffer(raw.length);
    const out = new Uint8Array(buffer);
    for (let i = 0; i < raw.length; ++i) out[i] = raw.charCodeAt(i);
    return out;
}

/// Convert a raw ArrayBuffer key from `getKey()` into base64url-no-padding,
/// which is what the backend expects (matches the format Chrome/Firefox use
/// in their JSON serialization of `PushSubscription`).
function arrayBufferToBase64Url(buf: ArrayBuffer | null): string {
    if (!buf) return '';
    const bytes = new Uint8Array(buf);
    let binary = '';
    for (let i = 0; i < bytes.byteLength; i++) binary += String.fromCharCode(bytes[i]);
    return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

export async function ensureNotificationPermission(): Promise<NotificationPermission> {
    if (Notification.permission === 'granted' || Notification.permission === 'denied') {
        return Notification.permission;
    }
    return await Notification.requestPermission();
}

export async function getServiceWorkerRegistration(): Promise<ServiceWorkerRegistration> {
    const reg = await navigator.serviceWorker.ready;
    if (!reg) throw new Error('Service worker not registered');
    return reg;
}

/// Subscribe this browser to push notifications and register the
/// subscription with the backend. Idempotent: if the browser already has a
/// matching subscription we just re-POST it (the server upserts by endpoint).
export async function enablePush(): Promise<void> {
    if (!isPushSupported()) {
        throw new Error('Push notifications worden niet ondersteund in deze browser.');
    }

    const permission = await ensureNotificationPermission();
    if (permission !== 'granted') {
        throw new Error('Toestemming voor meldingen geweigerd.');
    }

    const reg = await getServiceWorkerRegistration();
    const vapidKey = await getVapidPublicKey();

    let sub = await reg.pushManager.getSubscription();
    if (!sub) {
        sub = await reg.pushManager.subscribe({
            userVisibleOnly: true,
            applicationServerKey: urlBase64ToUint8Array(vapidKey),
        });
    }

    const p256dh = arrayBufferToBase64Url(sub.getKey('p256dh'));
    const auth = arrayBufferToBase64Url(sub.getKey('auth'));

    await subscribePush({
        endpoint: sub.endpoint,
        p256dh,
        auth,
        notifyGoal: true,
    });
}

export async function disablePush(): Promise<void> {
    if (!isPushSupported()) return;

    const reg = await getServiceWorkerRegistration();
    const sub = await reg.pushManager.getSubscription();
    if (!sub) return;

    try {
        await unsubscribePush(sub.endpoint);
    } finally {
        await sub.unsubscribe();
    }
}

export async function isCurrentBrowserSubscribed(): Promise<boolean> {
    if (!isPushSupported()) return false;
    try {
        const reg = await getServiceWorkerRegistration();
        const sub = await reg.pushManager.getSubscription();
        if (!sub) return false;
        // Cross-check with the server: a stale browser-side sub is meaningless
        // if the backend already pruned it.
        const serverSubs = await listMyPushSubscriptions();
        return serverSubs.some((s) => s.endpoint === sub.endpoint);
    } catch {
        return false;
    }
}

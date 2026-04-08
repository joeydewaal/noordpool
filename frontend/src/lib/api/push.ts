import type { PushSubscriptionRecord, SubscribeRequest } from './types';
import { api } from './client';

export async function getVapidPublicKey(): Promise<string> {
    const res = await api.get<{ key: string }>('/push/vapid-public-key');
    return res.data.key;
}

export async function subscribePush(data: SubscribeRequest): Promise<PushSubscriptionRecord> {
    return (await api.post<PushSubscriptionRecord>('/push/subscriptions', data)).data;
}

export async function unsubscribePush(endpoint: string): Promise<void> {
    await api.delete('/push/subscriptions', { data: { endpoint } });
}

export async function listMyPushSubscriptions(): Promise<PushSubscriptionRecord[]> {
    return (await api.get<PushSubscriptionRecord[]>('/push/subscriptions/me')).data;
}

export async function updatePushPrefs(
    id: string,
    prefs: { notifyGoal?: boolean },
): Promise<PushSubscriptionRecord> {
    return (await api.patch<PushSubscriptionRecord>(`/push/subscriptions/${id}`, prefs)).data;
}

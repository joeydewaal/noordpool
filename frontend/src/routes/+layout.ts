import { me } from '$lib/api/auth.ts';
import { auth } from '$lib/state/auth.svelte.ts';
import { browser } from '$app/environment';

export async function load({ fetch }: { fetch: typeof globalThis.fetch }) {
	if (browser) {
		try {
			const user = await me(fetch);
			if (user) {
				auth.setUser(user);
			}
		} finally {
			auth.loading = false;
		}
	}
}

export const ssr = false;

import { me } from '$lib/api/auth.js';
import { auth } from '$lib/state/auth.svelte.js';
import { browser } from '$app/environment';

export async function load() {
	if (browser) {
		try {
			const user = await me();
			if (user) {
				auth.setUser(user);
			}
		} finally {
			auth.loading = false;
		}
	}
}

export const ssr = false;

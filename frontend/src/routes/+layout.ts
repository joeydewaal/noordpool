import { me } from '$lib/api/auth';
import { setToken } from '$lib/api/client';
import { auth } from '$lib/state/auth.svelte';
import { browser } from '$app/environment';
import { redirect } from '@sveltejs/kit';

function parseJwt(token: string): Record<string, unknown> {
	const payload = token.split('.')[1];
	return JSON.parse(atob(payload.replace(/-/g, '+').replace(/_/g, '/')));
}

export async function load({ url }: { url: URL }) {
	if (!browser) {
		auth.loading = false;
		return;
	}

	const oauthToken = url.searchParams.get('token');
	if (oauthToken) {
		setToken(oauthToken);
		const claims = parseJwt(oauthToken);
		try {
			const user = await me();
			if (user) auth.setUser(user);
		} finally {
			auth.loading = false;
		}
		if (!claims.player_id) {
			const name = typeof claims.name === 'string' ? claims.name : '';
			redirect(302, `/auth/link-player?name=${encodeURIComponent(name)}`);
		}
		redirect(302, '/');
	}

	try {
		const user = await me();
		if (user) auth.setUser(user);
	} finally {
		auth.loading = false;
	}
}

export const ssr = false;

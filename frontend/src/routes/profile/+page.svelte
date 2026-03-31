<script lang="ts">
	import { auth } from '$lib/state/auth.svelte';
	import { pwa } from '$lib/state/pwa.svelte';
	import { theme } from '$lib/state/theme.svelte';
	import { logout, unlinkPlayer } from '$lib/api/auth';
	import { goto } from '$app/navigation';

	if (!auth.isAuthenticated) {
		goto('/auth/login');
	}

	let hasPlayer = $derived(auth.user?.roles.includes('player') ?? false);

	async function handleUnlink() {
		const res = await unlinkPlayer();
		auth.setUser(res.user);
	}

	function handleLogout() {
		logout();
		auth.clear();
		goto('/');
	}
</script>

{#if auth.isAuthenticated}
	<div class="max-w-md mx-auto p-6 space-y-6">
		<div class="card p-6 flex flex-col items-center gap-4">
			{#if auth.user?.avatarUrl}
				<img
					src={auth.user.avatarUrl}
					alt="{auth.user.firstName} {auth.user.lastName}"
					class="w-20 h-20 rounded-full object-cover"
				/>
			{:else}
				<div class="w-20 h-20 rounded-full bg-surface-500 flex items-center justify-center text-3xl font-bold text-white">
					{auth.user?.firstName?.charAt(0).toUpperCase()}
				</div>
			{/if}

			<div class="text-center">
				<h1 class="text-xl font-bold">{auth.user?.firstName} {auth.user?.lastName}</h1>
				<p class="text-surface-400">{auth.user?.email}</p>
			</div>

			{#if auth.user?.roles.length}
				<div class="flex flex-wrap gap-2 justify-center">
					{#each auth.user.roles as role}
						<span class="chip preset-filled-surface-500">{role}</span>
					{/each}
				</div>
			{/if}
		</div>

		{#if hasPlayer}
			<div class="card p-4 flex items-center justify-between">
				<span class="font-medium">Gekoppelde speler</span>
				<button class="btn btn-sm preset-filled-warning-500" onclick={handleUnlink}>
					Ontkoppelen
				</button>
			</div>
		{:else}
			<a href="/auth/link-player?name={encodeURIComponent(`${auth.user?.firstName ?? ''} ${auth.user?.lastName ?? ''}`.trim())}" class="card p-4 flex items-center justify-between">
				<span class="font-medium">Geen speler gekoppeld</span>
				<span class="btn btn-sm preset-filled-primary-500">Koppelen</span>
			</a>
		{/if}

		<div class="card p-4 flex items-center justify-between">
			<span class="font-medium">Thema</span>
			<button class="btn preset-filled-surface-500" onclick={() => theme.toggle()}>
				{#if theme.isDark}
					<svg class="w-5 h-5" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" d="M12 3v1m0 16v1m8.66-13.66l-.71.71M4.05 19.95l-.71.71M21 12h-1M4 12H3m16.66 7.66l-.71-.71M4.05 4.05l-.71-.71M16 12a4 4 0 11-8 0 4 4 0 018 0z" />
					</svg>
					<span>Licht thema</span>
				{:else}
					<svg class="w-5 h-5" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
						<path stroke-linecap="round" stroke-linejoin="round" d="M21 12.79A9 9 0 1111.21 3a7 7 0 009.79 9.79z" />
					</svg>
					<span>Donker thema</span>
				{/if}
			</button>
		</div>

		{#if pwa.installable}
			<button class="btn preset-filled-surface-500 w-full md:hidden" onclick={() => pwa.install()}>
				<svg class="w-5 h-5" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
					<path stroke-linecap="round" stroke-linejoin="round" d="M4 16v2a2 2 0 002 2h12a2 2 0 002-2v-2M7 10l5 5 5-5M12 15V3" />
				</svg>
				<span>App installeren</span>
			</button>
		{/if}

		<button class="btn preset-filled-error-500 w-full" onclick={handleLogout}>
			<svg class="w-5 h-5" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
				<path stroke-linecap="round" stroke-linejoin="round" d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" />
			</svg>
			<span>Uitloggen</span>
		</button>
	</div>
{/if}

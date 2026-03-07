<script lang="ts">
	import { onMount } from 'svelte';
	import { auth } from '$lib/state/auth.svelte.js';
	import { getPlayers } from '$lib/api/players.js';
	import type { Player } from '$lib/api/types.js';

	let players: Player[] = $state([]);
	let showInactive = $state(false);

	const canManage = $derived(auth.isAdmin || auth.isModerator);

	const filtered = $derived(
		showInactive ? players : players.filter((p) => p.active)
	);

	const positionColor: Record<string, string> = {
		goalkeeper: 'bg-amber-100 text-amber-800',
		defender: 'bg-blue-100 text-blue-800',
		midfielder: 'bg-green-100 text-green-800',
		forward: 'bg-red-100 text-red-800'
	};

	onMount(() => {
		players = getPlayers();
	});
</script>

<div class="flex items-center justify-between mb-6">
	<h1 class="text-2xl font-bold text-gray-900">Spelers</h1>
	<div class="flex items-center gap-3">
		{#if canManage}
			<label class="flex items-center gap-2 text-sm text-gray-600">
				<input type="checkbox" bind:checked={showInactive} class="rounded" />
				Toon inactief
			</label>
			<a
				href="/players/new"
				class="bg-primary hover:bg-primary-light text-white text-sm font-medium px-4 py-2 rounded-lg transition-colors"
			>
				Speler toevoegen
			</a>
		{/if}
	</div>
</div>

<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
	{#each filtered as player}
		<a
			href="/players/{player.id}"
			class="bg-white rounded-lg shadow hover:shadow-md transition-shadow p-5 flex items-center gap-4 {!player.active ? 'opacity-60' : ''}"
		>
			<div class="text-2xl font-bold text-primary w-12 text-center">
				{player.shirtNumber}
			</div>
			<div class="flex-1 min-w-0">
				<div class="font-semibold text-gray-900 truncate">{player.name}</div>
				<span class="inline-block mt-1 text-xs font-medium px-2 py-0.5 rounded-full {positionColor[player.position]}">
					{player.position}
				</span>
				{#if !player.active}
					<span class="inline-block mt-1 ml-1 text-xs font-medium px-2 py-0.5 rounded-full bg-gray-100 text-gray-500">
						inactief
					</span>
				{/if}
			</div>
		</a>
	{/each}
</div>

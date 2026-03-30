<script lang="ts">
	import { findPlayer, linkPlayer } from '$lib/api/auth';
	import { auth } from '$lib/state/auth.svelte';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import type { PlayerMatch } from '$lib/api/types';

	let playerMatches: PlayerMatch[] = $state([]);
	let loading = $state(true);

	const name = $derived($page.url.searchParams.get('name') ?? '');

	onMount(async () => {
		if (name.length >= 2) {
			try {
				playerMatches = await findPlayer(name);
			} catch {
				// silently ignore
			}
		}
		loading = false;
	});

	async function confirmLink(player: PlayerMatch) {
		try {
			const res = await linkPlayer(player.id);
			auth.setUser(res.user);
		} catch {
			// ignore — player may already be linked, just continue
		}
		goto('/');
	}
</script>

<div class="max-w-md mx-auto mt-8">
	<h1 class="text-2xl font-bold mb-2">Koppel je spelersprofiel</h1>
	<p class="text-surface-400 text-sm mb-6">Ben jij een van de spelers hieronder? Koppel je account om statistieken bij te houden.</p>

	{#if loading}
		<p class="text-sm text-surface-400">Laden...</p>
	{:else if playerMatches.length > 0}
		<div class="card preset-tonal-warning p-4 mb-4 space-y-3">
			<p class="text-sm font-medium">We vonden de volgende spelers. Ben jij dit?</p>
			{#each playerMatches as match}
				<div class="flex items-center justify-between">
					<span class="text-sm"><strong>{match.name}</strong> (#{match.shirtNumber}, {match.position})</span>
					<button type="button" onclick={() => confirmLink(match)} class="btn btn-sm preset-filled-primary-500 ml-3">
						Ja, dat ben ik
					</button>
				</div>
			{/each}
		</div>
	{:else}
		<div class="card preset-tonal p-4 mb-4 text-sm text-surface-400">
			Geen overeenkomende spelers gevonden.
		</div>
	{/if}

	<button type="button" onclick={() => goto('/')} class="btn w-full preset-outlined-surface-500">
		Overslaan
	</button>
</div>

<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { auth } from '$lib/state/auth.svelte.js';
	import { getPlayer, updatePlayer } from '$lib/api/players.js';
	import { getPlayerStats } from '$lib/api/events.js';
	import type { Player, PlayerStats } from '$lib/api/types.js';

	let player: Player | null = $state(null);
	let stats: PlayerStats | null = $state(null);

	const canManage = $derived(auth.isAdmin || auth.isModerator);

	const positionColor: Record<string, string> = {
		goalkeeper: 'preset-filled-warning-500',
		defender: 'preset-filled-secondary-500',
		midfielder: 'preset-filled-primary-500',
		forward: 'preset-filled-error-500'
	};

	async function toggleActive() {
		if (!player) return;
		player = await updatePlayer(player.id, { active: !player.active });
	}

	onMount(async () => {
		player = await getPlayer(page.params.id);
		if (player) stats = await getPlayerStats(player.id);
	});
</script>

{#if player}
	<div class="max-w-lg">
		<a href="/players" class="text-sm text-primary-500 hover:underline mb-4 inline-block">&larr; Alle spelers</a>
		<div class="card p-6">
			<div class="flex items-center gap-4 mb-4">
				<div class="text-4xl font-bold text-primary-500">{player.shirtNumber}</div>
				<div>
					<h1 class="text-2xl font-bold">{player.name}</h1>
					<span class="chip mt-1 {positionColor[player.position]}">
						{player.position}
					</span>
					{#if !player.active}
						<span class="chip mt-1 ml-1 preset-tonal-surface">
							inactief
						</span>
					{/if}
				</div>
			</div>

			{#if stats}
				<div class="grid grid-cols-5 gap-2 mt-4">
					{#each [
						{ value: stats.appearances, label: 'Wedstrijden' },
						{ value: stats.goals, label: 'Doelpunten' },
						{ value: stats.assists, label: 'Assists' },
						{ value: stats.yellowCards, label: 'Gele kaarten' },
						{ value: stats.redCards, label: 'Rode kaarten' }
					] as item}
						<div class="text-center p-2 card preset-tonal-surface rounded-lg">
							<div class="text-2xl font-bold">{item.value}</div>
							<div class="text-xs text-surface-400 mt-1">{item.label}</div>
						</div>
					{/each}
				</div>
			{/if}

			{#if canManage}
				<div class="flex gap-3 mt-6 pt-4 border-t border-surface-200 dark:border-surface-800">
					<a
						href="/players/{player.id}/edit"
						class="btn btn-sm preset-filled-primary-500"
					>
						Bewerken
					</a>
					<button
						onclick={toggleActive}
						class="btn btn-sm preset-outlined-surface-500"
					>
						{player.active ? 'Deactiveren' : 'Activeren'}
					</button>
				</div>
			{/if}
		</div>
	</div>
{:else}
	<p class="text-surface-400">Speler niet gevonden.</p>
{/if}

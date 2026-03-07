<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { auth } from '$lib/state/auth.svelte.js';
	import { getPlayer, updatePlayer } from '$lib/api/players.js';
	import type { Player } from '$lib/api/types.js';

	let player: Player | null = $state(null);

	const canManage = $derived(auth.isAdmin || auth.isModerator);

	const positionColor: Record<string, string> = {
		goalkeeper: 'bg-amber-100 text-amber-800',
		defender: 'bg-blue-100 text-blue-800',
		midfielder: 'bg-green-100 text-green-800',
		forward: 'bg-red-100 text-red-800'
	};

	function toggleActive() {
		if (!player) return;
		player = updatePlayer(player.id, { active: !player.active });
	}

	onMount(() => {
		player = getPlayer(page.params.id);
	});
</script>

{#if player}
	<div class="max-w-lg">
		<a href="/players" class="text-sm text-primary hover:underline mb-4 inline-block">&larr; Alle spelers</a>
		<div class="bg-white rounded-lg shadow p-6">
			<div class="flex items-center gap-4 mb-4">
				<div class="text-4xl font-bold text-primary">{player.shirtNumber}</div>
				<div>
					<h1 class="text-2xl font-bold text-gray-900">{player.name}</h1>
					<span class="inline-block mt-1 text-sm font-medium px-2.5 py-0.5 rounded-full {positionColor[player.position]}">
						{player.position}
					</span>
					{#if !player.active}
						<span class="inline-block mt-1 ml-1 text-sm font-medium px-2.5 py-0.5 rounded-full bg-gray-100 text-gray-500">
							inactief
						</span>
					{/if}
				</div>
			</div>

			<p class="text-sm text-gray-400 mt-4">Statistieken volgen later</p>

			{#if canManage}
				<div class="flex gap-3 mt-6 pt-4 border-t border-gray-100">
					<a
						href="/players/{player.id}/edit"
						class="bg-primary hover:bg-primary-light text-white text-sm font-medium px-4 py-2 rounded-lg transition-colors"
					>
						Bewerken
					</a>
					<button
						onclick={toggleActive}
						class="text-sm font-medium px-4 py-2 rounded-lg border border-gray-300 hover:bg-gray-50 transition-colors"
					>
						{player.active ? 'Deactiveren' : 'Activeren'}
					</button>
				</div>
			{/if}
		</div>
	</div>
{:else}
	<p class="text-gray-500">Speler niet gevonden.</p>
{/if}

<script lang="ts">
	import { onMount } from 'svelte';
	import { getLeaderboard } from '$lib/api/events.js';
	import type { LeaderboardEntry } from '$lib/api/types.js';

	let topScorers: LeaderboardEntry[] = $state([]);
	let topAssisters: LeaderboardEntry[] = $state([]);
	let mostCarded: LeaderboardEntry[] = $state([]);

	onMount(async () => {
		const data = await getLeaderboard();
		topScorers = data.topScorers;
		topAssisters = data.topAssisters;
		mostCarded = data.mostCarded;
	});
</script>

<div class="max-w-2xl space-y-6">
	<h1 class="text-2xl font-bold text-gray-900">Statistieken</h1>

	<div class="bg-white rounded-lg shadow p-6">
		<h2 class="text-lg font-bold text-gray-900 mb-3">Topscorers</h2>
		{#if topScorers.length === 0}
			<p class="text-sm text-gray-400">Nog geen doelpunten.</p>
		{:else}
			<table class="w-full text-sm">
				<thead>
					<tr class="text-left text-gray-500 border-b">
						<th class="pb-2 w-8">#</th>
						<th class="pb-2">Speler</th>
						<th class="pb-2 w-12 text-center">Nr</th>
						<th class="pb-2 w-16 text-center">Goals</th>
					</tr>
				</thead>
				<tbody>
					{#each topScorers as player, i}
						<tr class="border-b border-gray-50">
							<td class="py-2 text-gray-400">{i + 1}</td>
							<td class="py-2"><a href="/players/{player.playerId}" class="text-primary hover:underline font-medium">{player.name}</a></td>
							<td class="py-2 text-center text-gray-500">{player.shirtNumber}</td>
							<td class="py-2 text-center font-bold">{player.goals}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		{/if}
	</div>

	<div class="bg-white rounded-lg shadow p-6">
		<h2 class="text-lg font-bold text-gray-900 mb-3">Meeste assists</h2>
		{#if topAssisters.length === 0}
			<p class="text-sm text-gray-400">Nog geen assists.</p>
		{:else}
			<table class="w-full text-sm">
				<thead>
					<tr class="text-left text-gray-500 border-b">
						<th class="pb-2 w-8">#</th>
						<th class="pb-2">Speler</th>
						<th class="pb-2 w-12 text-center">Nr</th>
						<th class="pb-2 w-16 text-center">Assists</th>
					</tr>
				</thead>
				<tbody>
					{#each topAssisters as player, i}
						<tr class="border-b border-gray-50">
							<td class="py-2 text-gray-400">{i + 1}</td>
							<td class="py-2"><a href="/players/{player.playerId}" class="text-primary hover:underline font-medium">{player.name}</a></td>
							<td class="py-2 text-center text-gray-500">{player.shirtNumber}</td>
							<td class="py-2 text-center font-bold">{player.assists}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		{/if}
	</div>

	<div class="bg-white rounded-lg shadow p-6">
		<h2 class="text-lg font-bold text-gray-900 mb-3">Meeste kaarten</h2>
		{#if mostCarded.length === 0}
			<p class="text-sm text-gray-400">Nog geen kaarten.</p>
		{:else}
			<table class="w-full text-sm">
				<thead>
					<tr class="text-left text-gray-500 border-b">
						<th class="pb-2 w-8">#</th>
						<th class="pb-2">Speler</th>
						<th class="pb-2 w-12 text-center">Nr</th>
						<th class="pb-2 w-12 text-center">Geel</th>
						<th class="pb-2 w-12 text-center">Rood</th>
						<th class="pb-2 w-16 text-center">Totaal</th>
					</tr>
				</thead>
				<tbody>
					{#each mostCarded as player, i}
						<tr class="border-b border-gray-50">
							<td class="py-2 text-gray-400">{i + 1}</td>
							<td class="py-2"><a href="/players/{player.playerId}" class="text-primary hover:underline font-medium">{player.name}</a></td>
							<td class="py-2 text-center text-gray-500">{player.shirtNumber}</td>
							<td class="py-2 text-center">{player.yellowCards}</td>
							<td class="py-2 text-center">{player.redCards}</td>
							<td class="py-2 text-center font-bold">{player.totalCards}</td>
						</tr>
					{/each}
				</tbody>
			</table>
		{/if}
	</div>
</div>

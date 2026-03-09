<script lang="ts">
	import { onMount } from 'svelte';
	import { auth } from '$lib/state/auth.svelte.js';
	import { getUpcomingMatches, getRecentResults } from '$lib/api/matches.js';
	import type { Match } from '$lib/api/types.js';

	let upcoming: Match[] = $state([]);
	let results: Match[] = $state([]);
	let activeTab: 'upcoming' | 'results' = $state('upcoming');

	const canManage = $derived(auth.isAdmin || auth.isModerator);

	function formatDate(dateTime: string): string {
		return new Date(dateTime).toLocaleDateString('nl-NL', {
			weekday: 'short',
			day: 'numeric',
			month: 'short',
			year: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
	}

	function formatScore(match: Match): string {
		if (match.homeScore === null || (match.status === 'scheduled' && match.homeScore === 0)) return '';
		return `${match.homeScore} - ${match.awayScore}`;
	}

	onMount(async () => {
		[upcoming, results] = await Promise.all([
			getUpcomingMatches(),
			getRecentResults()
		]);
	});
</script>

<div class="flex items-center justify-between mb-6">
	<h1 class="text-2xl font-bold text-gray-900">Wedstrijden</h1>
	{#if canManage}
		<a
			href="/matches/new"
			class="bg-primary hover:bg-primary-light text-white text-sm font-medium px-4 py-2 rounded-lg transition-colors"
		>
			Nieuwe wedstrijd
		</a>
	{/if}
</div>

<div class="flex gap-2 mb-6">
	<button
		onclick={() => (activeTab = 'upcoming')}
		class="px-4 py-2 rounded-lg text-sm font-medium transition-colors {activeTab === 'upcoming' ? 'bg-primary text-white' : 'bg-white text-gray-600 hover:bg-gray-100'}"
	>
		Komend
	</button>
	<button
		onclick={() => (activeTab = 'results')}
		class="px-4 py-2 rounded-lg text-sm font-medium transition-colors {activeTab === 'results' ? 'bg-primary text-white' : 'bg-white text-gray-600 hover:bg-gray-100'}"
	>
		Uitslagen
	</button>
</div>

{#if activeTab === 'upcoming'}
	{#if upcoming.length === 0}
		<p class="text-gray-500 text-sm">Geen komende wedstrijden gepland.</p>
	{:else}
		<div class="space-y-3">
			{#each upcoming as match}
				<a href="/matches/{match.id}" class="block bg-white rounded-lg shadow hover:shadow-md transition-shadow p-4">
					<div class="flex items-center justify-between">
						<div>
							<div class="font-semibold text-gray-900">vs {match.opponent}</div>
							<div class="text-sm text-gray-500 mt-1">{formatDate(match.dateTime)}</div>
						</div>
						<span class="text-xs font-medium px-2.5 py-1 rounded-full {match.homeAway === 'home' ? 'bg-green-100 text-green-800' : 'bg-purple-100 text-purple-800'}">
							{match.homeAway === 'home' ? 'thuis' : 'uit'}
						</span>
					</div>
				</a>
			{/each}
		</div>
	{/if}
{:else}
	{#if results.length === 0}
		<p class="text-gray-500 text-sm">Nog geen uitslagen.</p>
	{:else}
		<div class="space-y-3">
			{#each results as match}
				<a href="/matches/{match.id}" class="block bg-white rounded-lg shadow hover:shadow-md transition-shadow p-4">
					<div class="flex items-center justify-between">
						<div>
							<div class="font-semibold text-gray-900">vs {match.opponent}</div>
							<div class="text-sm text-gray-500 mt-1">{formatDate(match.dateTime)}</div>
						</div>
						<div class="flex items-center gap-3">
							<span class="font-bold text-gray-900">{formatScore(match)}</span>
							<span class="text-xs font-medium px-2.5 py-1 rounded-full {match.homeAway === 'home' ? 'bg-green-100 text-green-800' : 'bg-purple-100 text-purple-800'}">
								{match.homeAway === 'home' ? 'thuis' : 'uit'}
							</span>
						</div>
					</div>
				</a>
			{/each}
		</div>
	{/if}
{/if}

<script lang="ts">
	import { onMount } from 'svelte';
	import { auth } from '$lib/state/auth.svelte.js';
	import { getUpcomingGames, getRecentResults } from '$lib/api/games.js';
	import type { Game } from '$lib/api/types.js';
	import { Tabs } from '@skeletonlabs/skeleton-svelte';

	let upcoming: Game[] = $state([]);
	let results: Game[] = $state([]);
	let activeTab = $state('upcoming');

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

	function formatScore(game: Game): string {
		if (game.homeScore === null || (game.status === 'scheduled' && game.homeScore === 0)) return '';
		return `${game.homeScore} - ${game.awayScore}`;
	}

	onMount(async () => {
		[upcoming, results] = await Promise.all([
			getUpcomingGames(),
			getRecentResults()
		]);
	});
</script>

<div class="flex items-center justify-between mb-6">
	<h1 class="text-2xl font-bold">Wedstrijden</h1>
	{#if canManage}
		<a href="/games/new" class="btn btn-sm preset-filled-primary-500">
			Nieuwe wedstrijd
		</a>
	{/if}
</div>

<Tabs.Root value={activeTab} onValueChange={(v) => (activeTab = v)}>
	<Tabs.List class="mb-6">
		<Tabs.Trigger value="upcoming">Komend</Tabs.Trigger>
		<Tabs.Trigger value="results">Uitslagen</Tabs.Trigger>
	</Tabs.List>
	<Tabs.Content value="upcoming">
		{#if upcoming.length === 0}
			<p class="text-surface-400 text-sm">Geen komende wedstrijden gepland.</p>
		{:else}
			<div class="space-y-3">
				{#each upcoming as game}
					<a href="/games/{game.id}" class="block card preset-tonal-surface p-4 hover:preset-tonal-primary transition-colors">
						<div class="flex items-center justify-between">
							<div>
								<div class="font-semibold">vs {game.opponent}</div>
								<div class="text-sm text-surface-400 mt-1">{formatDate(game.dateTime)}</div>
							</div>
							<span class="chip {game.homeAway === 'home' ? 'preset-filled-success-500' : 'preset-filled-secondary-500'}">
								{game.homeAway === 'home' ? 'thuis' : 'uit'}
							</span>
						</div>
					</a>
				{/each}
			</div>
		{/if}
	</Tabs.Content>
	<Tabs.Content value="results">
		{#if results.length === 0}
			<p class="text-surface-400 text-sm">Nog geen uitslagen.</p>
		{:else}
			<div class="space-y-3">
				{#each results as game}
					<a href="/games/{game.id}" class="block card preset-tonal-surface p-4 hover:preset-tonal-primary transition-colors">
						<div class="flex items-center justify-between">
							<div>
								<div class="font-semibold">vs {game.opponent}</div>
								<div class="text-sm text-surface-400 mt-1">{formatDate(game.dateTime)}</div>
							</div>
							<div class="flex items-center gap-3">
								<span class="font-bold">{formatScore(game)}</span>
								<span class="chip {game.homeAway === 'home' ? 'preset-filled-success-500' : 'preset-filled-secondary-500'}">
									{game.homeAway === 'home' ? 'thuis' : 'uit'}
								</span>
							</div>
						</div>
					</a>
				{/each}
			</div>
		{/if}
	</Tabs.Content>
</Tabs.Root>

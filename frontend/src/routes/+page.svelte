<script lang="ts">
	import { onMount } from 'svelte';
	import { auth } from '$lib/state/auth.svelte.js';
	import { getUpcomingMatches, getRecentResults } from '$lib/api/matches.js';
	import type { Match } from '$lib/api/types.js';

	let upcoming: Match[] = $state([]);
	let results: Match[] = $state([]);

	function formatDate(dateTime: string): string {
		return new Date(dateTime).toLocaleDateString('nl-NL', {
			weekday: 'short',
			day: 'numeric',
			month: 'short',
			hour: '2-digit',
			minute: '2-digit'
		});
	}

	function formatScore(match: Match): string {
		if (match.homeScore === null) return '';
		return `${match.homeScore} - ${match.awayScore}`;
	}

	onMount(() => {
		upcoming = getUpcomingMatches(3);
		results = getRecentResults(3);
	});
</script>

{#if auth.isAuthenticated}
	<div>
		<h1 class="text-2xl font-bold text-gray-900 mb-4">Welkom terug, {auth.user?.name}!</h1>
		<div class="grid gap-6 md:grid-cols-2">
			<div class="bg-white rounded-lg shadow p-6">
				<div class="flex items-center justify-between mb-3">
					<h2 class="text-lg font-semibold text-gray-800">Komende wedstrijden</h2>
					<a href="/matches" class="text-sm text-primary hover:underline">Bekijk alles</a>
				</div>
				{#if upcoming.length === 0}
					<p class="text-gray-500 text-sm">Geen komende wedstrijden.</p>
				{:else}
					<div class="space-y-3">
						{#each upcoming as match}
							<a href="/matches/{match.id}" class="block p-3 rounded-lg hover:bg-gray-50 transition-colors -mx-1">
								<div class="flex items-center justify-between">
									<div class="font-medium text-gray-900 text-sm">vs {match.opponent}</div>
									<span class="text-xs font-medium px-2 py-0.5 rounded-full {match.homeAway === 'home' ? 'bg-green-100 text-green-800' : 'bg-purple-100 text-purple-800'}">
										{match.homeAway}
									</span>
								</div>
								<div class="text-xs text-gray-500 mt-1">{formatDate(match.dateTime)}</div>
							</a>
						{/each}
					</div>
				{/if}
			</div>
			<div class="bg-white rounded-lg shadow p-6">
				<div class="flex items-center justify-between mb-3">
					<h2 class="text-lg font-semibold text-gray-800">Recente uitslagen</h2>
					<a href="/matches" class="text-sm text-primary hover:underline">Bekijk alles</a>
				</div>
				{#if results.length === 0}
					<p class="text-gray-500 text-sm">Nog geen uitslagen.</p>
				{:else}
					<div class="space-y-3">
						{#each results as match}
							<a href="/matches/{match.id}" class="block p-3 rounded-lg hover:bg-gray-50 transition-colors -mx-1">
								<div class="flex items-center justify-between">
									<div class="font-medium text-gray-900 text-sm">vs {match.opponent}</div>
									<div class="flex items-center gap-2">
										<span class="font-bold text-sm text-gray-900">{formatScore(match)}</span>
										<span class="text-xs font-medium px-2 py-0.5 rounded-full {match.homeAway === 'home' ? 'bg-green-100 text-green-800' : 'bg-purple-100 text-purple-800'}">
											{match.homeAway}
										</span>
									</div>
								</div>
								<div class="text-xs text-gray-500 mt-1">{formatDate(match.dateTime)}</div>
							</a>
						{/each}
					</div>
				{/if}
			</div>
		</div>
	</div>
{:else}
	<div class="text-center py-16">
		<h1 class="text-4xl font-bold text-gray-900 mb-4">Noordpool</h1>
		<p class="text-lg text-gray-600 mb-8">Welkom bij de Noordpool voetbal app.</p>
		<a
			href="/auth/login"
			class="inline-block bg-primary hover:bg-primary-light text-white font-medium px-8 py-3 rounded-lg transition-colors"
		>
			Aan de slag
		</a>
	</div>
	<div class="mt-12 grid gap-6 md:grid-cols-2">
		<div class="bg-white rounded-lg shadow p-6">
			<div class="flex items-center justify-between mb-3">
				<h2 class="text-lg font-semibold text-gray-800">Komende wedstrijden</h2>
				<a href="/matches" class="text-sm text-primary hover:underline">Bekijk alles</a>
			</div>
			{#if upcoming.length === 0}
				<p class="text-gray-500 text-sm">Geen komende wedstrijden.</p>
			{:else}
				<div class="space-y-3">
					{#each upcoming as match}
						<a href="/matches/{match.id}" class="block p-3 rounded-lg hover:bg-gray-50 transition-colors -mx-1">
							<div class="flex items-center justify-between">
								<div class="font-medium text-gray-900 text-sm">vs {match.opponent}</div>
								<span class="text-xs font-medium px-2 py-0.5 rounded-full {match.homeAway === 'home' ? 'bg-green-100 text-green-800' : 'bg-purple-100 text-purple-800'}">
									{match.homeAway === 'home' ? 'thuis' : 'uit'}
								</span>
							</div>
							<div class="text-xs text-gray-500 mt-1">{formatDate(match.dateTime)}</div>
						</a>
					{/each}
				</div>
			{/if}
		</div>
		<div class="bg-white rounded-lg shadow p-6">
			<div class="flex items-center justify-between mb-3">
				<h2 class="text-lg font-semibold text-gray-800">Recente uitslagen</h2>
				<a href="/matches" class="text-sm text-primary hover:underline">Bekijk alles</a>
			</div>
			{#if results.length === 0}
				<p class="text-gray-500 text-sm">Nog geen uitslagen.</p>
			{:else}
				<div class="space-y-3">
					{#each results as match}
						<a href="/matches/{match.id}" class="block p-3 rounded-lg hover:bg-gray-50 transition-colors -mx-1">
							<div class="flex items-center justify-between">
								<div class="font-medium text-gray-900 text-sm">vs {match.opponent}</div>
								<div class="flex items-center gap-2">
									<span class="font-bold text-sm text-gray-900">{formatScore(match)}</span>
									<span class="text-xs font-medium px-2 py-0.5 rounded-full {match.homeAway === 'home' ? 'bg-green-100 text-green-800' : 'bg-purple-100 text-purple-800'}">
										{match.homeAway === 'home' ? 'thuis' : 'uit'}
									</span>
								</div>
							</div>
							<div class="text-xs text-gray-500 mt-1">{formatDate(match.dateTime)}</div>
						</a>
					{/each}
				</div>
			{/if}
		</div>
	</div>
{/if}

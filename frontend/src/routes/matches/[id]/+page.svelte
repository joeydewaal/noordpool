<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { auth } from '$lib/state/auth.svelte.js';
	import { getMatch } from '$lib/api/matches.js';
	import type { Match } from '$lib/api/types.js';

	let match: Match | null = $state(null);

	const canManage = $derived(auth.isAdmin || auth.isModerator);

	function formatDate(dateTime: string): string {
		return new Date(dateTime).toLocaleDateString('nl-NL', {
			weekday: 'long',
			day: 'numeric',
			month: 'long',
			year: 'numeric',
			hour: '2-digit',
			minute: '2-digit'
		});
	}

	onMount(() => {
		match = getMatch(page.params.id);
	});
</script>

{#if match}
	<div class="max-w-lg">
		<a href="/matches" class="text-sm text-primary hover:underline mb-4 inline-block">&larr; Alle wedstrijden</a>
		<div class="bg-white rounded-lg shadow p-6">
			<div class="flex items-center justify-between mb-2">
				<h1 class="text-2xl font-bold text-gray-900">vs {match.opponent}</h1>
				<span class="text-xs font-medium px-2.5 py-1 rounded-full {match.homeAway === 'home' ? 'bg-green-100 text-green-800' : 'bg-purple-100 text-purple-800'}">
					{match.homeAway === 'home' ? 'thuis' : 'uit'}
				</span>
			</div>

			<div class="text-sm text-gray-500 space-y-1 mb-4">
				<div>{formatDate(match.dateTime)}</div>
				<div>{match.location}</div>
				<div>
					Status:
					<span class="font-medium {match.status === 'completed' ? 'text-green-700' : match.status === 'cancelled' ? 'text-red-700' : 'text-blue-700'}">
						{match.status === 'scheduled' ? 'gepland' : match.status === 'completed' ? 'gespeeld' : 'afgelast'}
					</span>
				</div>
			</div>

			{#if match.status === 'completed' && match.homeScore !== null}
				<div class="bg-gray-50 rounded-lg p-4 text-center">
					{#if match.homeAway === 'home'}
						<div class="text-lg">
							<span class="font-bold">Noordpool {match.homeScore}</span>
							<span class="text-gray-400 mx-2">-</span>
							<span class="font-bold">{match.awayScore} {match.opponent}</span>
						</div>
					{:else}
						<div class="text-lg">
							<span class="font-bold">{match.opponent} {match.homeScore}</span>
							<span class="text-gray-400 mx-2">-</span>
							<span class="font-bold">{match.awayScore} Noordpool</span>
						</div>
					{/if}
				</div>
			{/if}

			{#if canManage}
				<div class="mt-6 pt-4 border-t border-gray-100">
					<a
						href="/matches/{match.id}/edit"
						class="bg-primary hover:bg-primary-light text-white text-sm font-medium px-4 py-2 rounded-lg transition-colors"
					>
						Bewerken
					</a>
				</div>
			{/if}
		</div>
	</div>
{:else}
	<p class="text-gray-500">Wedstrijd niet gevonden.</p>
{/if}

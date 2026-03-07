<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { auth } from '$lib/state/auth.svelte.js';
	import { getMatch, updateMatch } from '$lib/api/matches.js';
	import type { HomeAway, MatchStatus } from '$lib/api/types.js';

	const canManage = $derived(auth.isAdmin || auth.isModerator);

	let opponent = $state('');
	let location = $state('');
	let dateTime = $state('');
	let homeAway: HomeAway = $state('home');
	let status: MatchStatus = $state('scheduled');
	let homeScore: number | null = $state(null);
	let awayScore: number | null = $state(null);
	let loaded = $state(false);

	function handleSubmit(e: Event) {
		e.preventDefault();
		updateMatch(page.params.id, {
			opponent,
			location,
			dateTime,
			homeAway,
			status,
			homeScore: status === 'completed' ? homeScore : null,
			awayScore: status === 'completed' ? awayScore : null
		});
		goto(`/matches/${page.params.id}`);
	}

	onMount(() => {
		const match = getMatch(page.params.id);
		if (match) {
			opponent = match.opponent;
			location = match.location;
			dateTime = match.dateTime;
			homeAway = match.homeAway;
			status = match.status;
			homeScore = match.homeScore;
			awayScore = match.awayScore;
			loaded = true;
		}
	});
</script>

{#if !canManage}
	<p class="text-red-600 font-medium">Geen toegang. Admin- of moderatorrol vereist.</p>
{:else if !loaded}
	<p class="text-gray-500">Wedstrijd niet gevonden.</p>
{:else}
	<div class="max-w-lg">
		<a href="/matches/{page.params.id}" class="text-sm text-primary hover:underline mb-4 inline-block">&larr; Terug naar wedstrijd</a>
		<h1 class="text-2xl font-bold text-gray-900 mb-6">Wedstrijd bewerken</h1>
		<form onsubmit={handleSubmit} class="bg-white rounded-lg shadow p-6 space-y-4">
			<div>
				<label for="opponent" class="block text-sm font-medium text-gray-700 mb-1">Tegenstander</label>
				<input
					id="opponent"
					type="text"
					bind:value={opponent}
					required
					class="w-full border border-gray-300 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-primary/50"
				/>
			</div>
			<div>
				<label for="location" class="block text-sm font-medium text-gray-700 mb-1">Locatie</label>
				<input
					id="location"
					type="text"
					bind:value={location}
					required
					class="w-full border border-gray-300 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-primary/50"
				/>
			</div>
			<div>
				<label for="dateTime" class="block text-sm font-medium text-gray-700 mb-1">Datum & tijd</label>
				<input
					id="dateTime"
					type="datetime-local"
					bind:value={dateTime}
					required
					class="w-full border border-gray-300 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-primary/50"
				/>
			</div>
			<fieldset>
				<legend class="block text-sm font-medium text-gray-700 mb-2">Thuis / Uit</legend>
				<div class="flex gap-4">
					<label class="flex items-center gap-2">
						<input type="radio" bind:group={homeAway} value="home" />
						<span class="text-sm">Thuis</span>
					</label>
					<label class="flex items-center gap-2">
						<input type="radio" bind:group={homeAway} value="away" />
						<span class="text-sm">Uit</span>
					</label>
				</div>
			</fieldset>
			<div>
				<label for="status" class="block text-sm font-medium text-gray-700 mb-1">Status</label>
				<select
					id="status"
					bind:value={status}
					class="w-full border border-gray-300 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-primary/50"
				>
					<option value="scheduled">Gepland</option>
					<option value="completed">Gespeeld</option>
					<option value="cancelled">Afgelast</option>
				</select>
			</div>
			{#if status === 'completed'}
				<div class="grid grid-cols-2 gap-4">
					<div>
						<label for="homeScore" class="block text-sm font-medium text-gray-700 mb-1">Thuisscore</label>
						<input
							id="homeScore"
							type="number"
							bind:value={homeScore}
							min="0"
							required
							class="w-full border border-gray-300 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-primary/50"
						/>
					</div>
					<div>
						<label for="awayScore" class="block text-sm font-medium text-gray-700 mb-1">Uitscore</label>
						<input
							id="awayScore"
							type="number"
							bind:value={awayScore}
							min="0"
							required
							class="w-full border border-gray-300 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-primary/50"
						/>
					</div>
				</div>
			{/if}
			<button
				type="submit"
				class="w-full bg-primary hover:bg-primary-light text-white font-medium py-2.5 rounded-lg transition-colors"
			>
				Wijzigingen opslaan
			</button>
		</form>
	</div>
{/if}

<script lang="ts">
	import { goto } from '$app/navigation';
	import { auth } from '$lib/state/auth.svelte.js';
	import { createMatch } from '$lib/api/matches.js';
	import type { HomeAway } from '$lib/api/types.js';

	const canManage = $derived(auth.isAdmin || auth.isModerator);

	let opponent = $state('');
	let location = $state('');
	let dateTime = $state('');
	let homeAway: HomeAway = $state('home');

	async function handleSubmit(e: Event) {
		e.preventDefault();
		await createMatch({ opponent, location, dateTime, homeAway });
		goto('/matches');
	}
</script>

{#if canManage}
	<div class="max-w-lg">
		<a href="/matches" class="text-sm text-primary hover:underline mb-4 inline-block">&larr; Alle wedstrijden</a>
		<h1 class="text-2xl font-bold text-gray-900 mb-6">Nieuwe wedstrijd</h1>
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
			<button
				type="submit"
				class="w-full bg-primary hover:bg-primary-light text-white font-medium py-2.5 rounded-lg transition-colors"
			>
				Wedstrijd aanmaken
			</button>
		</form>
	</div>
{:else}
	<p class="text-red-600 font-medium">Geen toegang. Admin- of moderatorrol vereist.</p>
{/if}

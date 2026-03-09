<script lang="ts">
	import { goto } from '$app/navigation';
	import { auth } from '$lib/state/auth.svelte.js';
	import { createPlayer } from '$lib/api/players.js';
	import type { Position } from '$lib/api/types.js';

	const canManage = $derived(auth.isAdmin || auth.isModerator);



	let name = $state('');
	let shirtNumber = $state(0);
	let position: Position = $state('midfielder');

	async function handleSubmit(e: Event) {
		e.preventDefault();
		await createPlayer({ name, shirtNumber, position });
		goto('/players');
	}
</script>

{#if canManage}
	<div class="max-w-lg">
		<a href="/players" class="text-sm text-primary hover:underline mb-4 inline-block">&larr; Alle spelers</a>
		<h1 class="text-2xl font-bold text-gray-900 mb-6">Speler toevoegen</h1>
		<form onsubmit={handleSubmit} class="bg-white rounded-lg shadow p-6 space-y-4">
			<div>
				<label for="name" class="block text-sm font-medium text-gray-700 mb-1">Naam</label>
				<input
					id="name"
					type="text"
					bind:value={name}
					required
					class="w-full border border-gray-300 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-primary/50"
				/>
			</div>
			<div>
				<label for="shirtNumber" class="block text-sm font-medium text-gray-700 mb-1">Rugnummer</label>
				<input
					id="shirtNumber"
					type="number"
					bind:value={shirtNumber}
					min="1"
					max="99"
					required
					class="w-full border border-gray-300 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-primary/50"
				/>
			</div>
			<div>
				<label for="position" class="block text-sm font-medium text-gray-700 mb-1">Positie</label>
				<select
					id="position"
					bind:value={position}
					class="w-full border border-gray-300 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-primary/50"
				>
					<option value="goalkeeper">Keeper</option>
					<option value="defender">Verdediger</option>
					<option value="midfielder">Middenvelder</option>
					<option value="forward">Aanvaller</option>
				</select>
			</div>
			<button
				type="submit"
				class="w-full bg-primary hover:bg-primary-light text-white font-medium py-2.5 rounded-lg transition-colors"
			>
				Speler aanmaken
			</button>
		</form>
	</div>
{:else}
	<p class="text-red-600 font-medium">Geen toegang. Admin- of moderatorrol vereist.</p>
{/if}

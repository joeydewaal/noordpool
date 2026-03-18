<script lang="ts">
	import { goto } from '$app/navigation';
	import { auth } from '$lib/state/auth.svelte';
	import { createPlayer } from '$lib/api/players';
	import { createMutation, useQueryClient } from '@tanstack/svelte-query';
	import type { Position, CreatePlayerRequest } from '$lib/api/types';

	const canManage = $derived(auth.isAdmin || auth.isModerator);
	const queryClient = useQueryClient();

	let name = $state('');
	let shirtNumber = $state(0);
	let position: Position = $state('midfielder');

	const createMut = createMutation(() => ({
		mutationFn: (data: CreatePlayerRequest) => createPlayer(data),
		onSuccess: () => {
			queryClient.invalidateQueries({ queryKey: ['players'] });
			goto('/players');
		},
	}));

	function handleSubmit(e: Event) {
		e.preventDefault();
		createMut.mutate({ name, shirtNumber, position });
	}
</script>

{#if canManage}
	<div class="max-w-lg">
		<a href="/players" class="text-sm text-primary-500 hover:underline mb-4 inline-block">&larr; Alle spelers</a>
		<h1 class="text-2xl font-bold mb-6">Speler toevoegen</h1>
		<form onsubmit={handleSubmit} class="card p-6 space-y-4">
			<div>
				<label for="name" class="label-text">Naam</label>
				<input id="name" type="text" bind:value={name} required class="input" />
			</div>
			<div>
				<label for="shirtNumber" class="label-text">Rugnummer</label>
				<input
					id="shirtNumber"
					type="number"
					bind:value={shirtNumber}
					min="1"
					max="99"
					required
					class="input"
				/>
			</div>
			<div>
				<label for="position" class="label-text">Positie</label>
				<select id="position" bind:value={position} class="select">
					<option value="goalkeeper">Keeper</option>
					<option value="defender">Verdediger</option>
					<option value="midfielder">Middenvelder</option>
					<option value="forward">Aanvaller</option>
				</select>
			</div>
			<button type="submit" class="btn w-full preset-filled-primary-500">
				Speler aanmaken
			</button>
		</form>
	</div>
{:else}
	<p class="text-error-500 font-medium">Geen toegang. Admin- of moderatorrol vereist.</p>
{/if}

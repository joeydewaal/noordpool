<script lang="ts">
	import { register, findPlayer } from '$lib/api/auth';
	import { auth } from '$lib/state/auth.svelte';
	import { goto } from '$app/navigation';
	import GoogleOAuthButton from '$lib/components/GoogleOAuthButton.svelte';
	import type { PlayerMatch } from '$lib/api/types';

	let name = $state('');
	let email = $state('');
	let password = $state('');
	let confirmPassword = $state('');
	let error = $state('');

	let playerMatches: PlayerMatch[] = $state([]);
	let linkedPlayer: PlayerMatch | null = $state(null);
	let dismissed = $state(false);

	async function onNameBlur() {
		if (name.trim().length < 2 || dismissed || linkedPlayer) return;
		try {
			playerMatches = await findPlayer(name.trim());
		} catch {
			// silently ignore
		}
	}

	function confirmLink(player: PlayerMatch) {
		linkedPlayer = player;
		playerMatches = [];
	}

	function dismissMatches() {
		dismissed = true;
		playerMatches = [];
	}

	function unlinkPlayer() {
		linkedPlayer = null;
		dismissed = false;
	}

	async function handleSubmit(e: Event) {
		e.preventDefault();
		error = '';

		if (password !== confirmPassword) {
			error = 'Wachtwoorden komen niet overeen.';
			return;
		}

		try {
			const res = await register({
				name,
				email,
				password,
				...(linkedPlayer ? { playerId: linkedPlayer.id } : {})
			});
			auth.setUser(res.user);
			goto('/');
		} catch (err) {
			error = 'Registratie mislukt. Probeer het opnieuw.';
		}
	}
</script>

<div class="max-w-md mx-auto mt-8">
	<h1 class="text-2xl font-bold mb-6">Registreren</h1>

	{#if error}
		<div class="card preset-tonal-error p-3 mb-4 text-sm">{error}</div>
	{/if}

	{#if linkedPlayer}
		<div class="card preset-tonal-success p-3 mb-4 flex items-center justify-between text-sm">
			<span>Gekoppeld aan <strong>{linkedPlayer.name}</strong> (#{linkedPlayer.shirtNumber})</span>
			<button type="button" onclick={unlinkPlayer} class="ml-2 text-xs underline opacity-70 hover:opacity-100">Ontkoppelen</button>
		</div>
	{/if}

	{#if playerMatches.length > 0}
		<div class="card preset-tonal-warning p-4 mb-4 space-y-3">
			<p class="text-sm font-medium">We vonden de volgende spelers. Ben jij dit?</p>
			{#each playerMatches as match}
				<div class="flex items-center justify-between">
					<span class="text-sm"><strong>{match.name}</strong> (#{match.shirtNumber}, {match.position})</span>
					<button type="button" onclick={() => confirmLink(match)} class="btn btn-sm preset-filled-primary-500 ml-3">
						Ja, dat ben ik
					</button>
				</div>
			{/each}
			<button type="button" onclick={dismissMatches} class="text-xs underline opacity-60 hover:opacity-100">
				Nee, ik ben dit niet
			</button>
		</div>
	{/if}

	<form onsubmit={handleSubmit} class="space-y-4">
		<div>
			<label for="name" class="label-text">Naam</label>
			<input id="name" type="text" bind:value={name} onblur={onNameBlur} required class="input" />
		</div>
		<div>
			<label for="email" class="label-text">E-mail</label>
			<input id="email" type="email" bind:value={email} required class="input" />
		</div>
		<div>
			<label for="password" class="label-text">Wachtwoord</label>
			<input id="password" type="password" bind:value={password} required class="input" />
		</div>
		<div>
			<label for="confirmPassword" class="label-text">Bevestig wachtwoord</label>
			<input id="confirmPassword" type="password" bind:value={confirmPassword} required class="input" />
		</div>
		<button type="submit" class="btn w-full preset-filled-primary-500">
			Registreren
		</button>
	</form>

	<div class="my-6 flex items-center gap-4">
		<hr class="flex-1 border-surface-300 dark:border-surface-700" />
		<span class="text-sm text-surface-400">of</span>
		<hr class="flex-1 border-surface-300 dark:border-surface-700" />
	</div>

	<GoogleOAuthButton />

	<p class="mt-6 text-center text-sm text-surface-400">
		Al een account? <a href="/auth/login" class="text-primary-500 font-medium hover:underline">Inloggen</a>
	</p>
</div>

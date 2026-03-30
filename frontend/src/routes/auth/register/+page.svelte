<script lang="ts">
	import { register } from '$lib/api/auth';
	import { auth } from '$lib/state/auth.svelte';
	import { goto } from '$app/navigation';
	import GoogleOAuthButton from '$lib/components/GoogleOAuthButton.svelte';

	let firstName = $state('');
	let lastName = $state('');
	let email = $state('');
	let password = $state('');
	let confirmPassword = $state('');
	let error = $state('');

	async function handleSubmit(e: Event) {
		e.preventDefault();
		error = '';

		if (password !== confirmPassword) {
			error = 'Wachtwoorden komen niet overeen.';
			return;
		}

		try {
			const res = await register({ firstName, lastName, email, password });
			auth.setUser(res.user);
			goto(`/auth/link-player?name=${encodeURIComponent(`${firstName} ${lastName}`)}`);
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

	<form onsubmit={handleSubmit} class="space-y-4">
		<div>
			<label for="firstName" class="label-text">Voornaam</label>
			<input id="firstName" type="text" bind:value={firstName} required class="input" />
		</div>
		<div>
			<label for="lastName" class="label-text">Achternaam</label>
			<input id="lastName" type="text" bind:value={lastName} required class="input" />
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

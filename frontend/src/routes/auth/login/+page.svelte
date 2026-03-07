<script lang="ts">
	import { login } from '$lib/api/auth.js';
	import { auth } from '$lib/state/auth.svelte.js';
	import { goto } from '$app/navigation';
	import GoogleOAuthButton from '$lib/components/GoogleOAuthButton.svelte';

	let email = $state('');
	let password = $state('');
	let error = $state('');

	async function handleSubmit(e: Event) {
		e.preventDefault();
		error = '';
		try {
			const res = await login({ email, password });
			auth.setUser(res.user);
			goto('/');
		} catch (err) {
			error = 'Login failed. Please try again.';
		}
	}
</script>

<div class="max-w-md mx-auto mt-8">
	<h1 class="text-2xl font-bold text-gray-900 mb-6">Login</h1>

	{#if error}
		<div class="bg-red-50 text-red-700 p-3 rounded mb-4 text-sm">{error}</div>
	{/if}

	<form onsubmit={handleSubmit} class="space-y-4">
		<div>
			<label for="email" class="block text-sm font-medium text-gray-700 mb-1">Email</label>
			<input
				id="email"
				type="email"
				bind:value={email}
				required
				class="w-full border border-gray-300 rounded px-3 py-2 focus:outline-none focus:ring-2 focus:ring-primary"
			/>
		</div>
		<div>
			<label for="password" class="block text-sm font-medium text-gray-700 mb-1">Password</label>
			<input
				id="password"
				type="password"
				bind:value={password}
				required
				class="w-full border border-gray-300 rounded px-3 py-2 focus:outline-none focus:ring-2 focus:ring-primary"
			/>
		</div>
		<button
			type="submit"
			class="w-full bg-primary hover:bg-primary-light text-white font-medium py-2.5 rounded transition-colors"
		>
			Login
		</button>
	</form>

	<div class="my-6 flex items-center gap-4">
		<hr class="flex-1 border-gray-300" />
		<span class="text-sm text-gray-500">or</span>
		<hr class="flex-1 border-gray-300" />
	</div>

	<GoogleOAuthButton />

	<p class="mt-6 text-center text-sm text-gray-600">
		Don't have an account? <a href="/auth/register" class="text-primary font-medium hover:underline">Register</a>
	</p>
</div>

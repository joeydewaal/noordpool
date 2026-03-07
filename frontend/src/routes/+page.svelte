<script lang="ts">
	import { auth } from '$lib/state/auth.svelte.js';
</script>

{#if auth.isAuthenticated}
	<div>
		<h1 class="text-2xl font-bold text-gray-900 mb-4">Welcome back, {auth.user?.name}!</h1>
		<div class="grid gap-6 md:grid-cols-2">
			<div class="bg-white rounded-lg shadow p-6">
				<h2 class="text-lg font-semibold text-gray-800 mb-2">Upcoming Matches</h2>
				<p class="text-gray-500 text-sm">No upcoming matches yet.</p>
			</div>
			<div class="bg-white rounded-lg shadow p-6">
				<h2 class="text-lg font-semibold text-gray-800 mb-2">Recent Results</h2>
				<p class="text-gray-500 text-sm">No recent results yet.</p>
			</div>
		</div>
	</div>
{:else}
	<div class="text-center py-16">
		<h1 class="text-4xl font-bold text-gray-900 mb-4">Noordpool</h1>
		<p class="text-lg text-gray-600 mb-8">Welcome to the Noordpool football team app.</p>
		<a
			href="/auth/login"
			class="inline-block bg-primary hover:bg-primary-light text-white font-medium px-8 py-3 rounded-lg transition-colors"
		>
			Get Started
		</a>
	</div>
{/if}

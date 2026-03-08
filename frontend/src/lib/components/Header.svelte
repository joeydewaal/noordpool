<script lang="ts">
	import { auth } from '$lib/state/auth.svelte.js';
	import { logout } from '$lib/api/auth.js';
	import { goto } from '$app/navigation';

	let mobileMenuOpen = $state(false);

	function handleLogout() {
		logout();
		auth.clear();
		goto('/');
	}
</script>

<header class="bg-primary text-white shadow-md">
	<div class="max-w-5xl mx-auto px-4 flex items-center justify-between h-16">
		<a href="/" class="text-xl font-bold tracking-tight">Noordpool</a>

		<!-- Desktop nav -->
		<nav class="hidden md:flex items-center gap-6">
			<a href="/" class="hover:text-accent-light transition-colors">Home</a>
			<a href="/matches" class="hover:text-accent-light transition-colors">Wedstrijden</a>
			<a href="/players" class="hover:text-accent-light transition-colors">Spelers</a>
			<a href="/stats" class="hover:text-accent-light transition-colors">Statistieken</a>
		</nav>

		<div class="hidden md:flex items-center gap-4">
			{#if auth.isAuthenticated}
				<span class="text-sm">{auth.user?.name}</span>
				<button
					onclick={handleLogout}
					class="text-sm bg-primary-light hover:bg-white/20 px-3 py-1.5 rounded transition-colors"
				>
					Uitloggen
				</button>
			{:else}
				<a
					href="/auth/login"
					class="text-sm bg-accent hover:bg-accent-light text-primary font-medium px-4 py-1.5 rounded transition-colors"
				>
					Inloggen
				</a>
			{/if}
		</div>

		<!-- Mobile hamburger -->
		<button
			class="md:hidden p-2"
			onclick={() => (mobileMenuOpen = !mobileMenuOpen)}
			aria-label="Menu openen"
		>
			<svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
				{#if mobileMenuOpen}
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
				{:else}
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
				{/if}
			</svg>
		</button>
	</div>

	<!-- Mobile menu -->
	{#if mobileMenuOpen}
		<nav class="md:hidden border-t border-white/20 px-4 py-4 flex flex-col gap-3">
			<a href="/" class="hover:text-accent-light" onclick={() => (mobileMenuOpen = false)}>Home</a>
			<a href="/matches" class="hover:text-accent-light" onclick={() => (mobileMenuOpen = false)}>Wedstrijden</a>
			<a href="/players" class="hover:text-accent-light" onclick={() => (mobileMenuOpen = false)}>Spelers</a>
			<a href="/stats" class="hover:text-accent-light" onclick={() => (mobileMenuOpen = false)}>Statistieken</a>
			<hr class="border-white/20" />
			{#if auth.isAuthenticated}
				<span class="text-sm">{auth.user?.name}</span>
				<button onclick={handleLogout} class="text-left text-sm hover:text-accent-light">Uitloggen</button>
			{:else}
				<a href="/auth/login" onclick={() => (mobileMenuOpen = false)} class="text-accent font-medium">Inloggen</a>
			{/if}
		</nav>
	{/if}
</header>

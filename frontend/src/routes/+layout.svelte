<script lang="ts">
	import '../app.css';
	import Header from '$lib/components/Header.svelte';

	let { children } = $props();
</script>

<div class="min-h-screen bg-gray-50 flex flex-col">
	<Header />
	<main class="flex-1 max-w-5xl mx-auto w-full px-4 py-8">
		{@render children()}
	</main>
</div>

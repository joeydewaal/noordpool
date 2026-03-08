<script lang="ts">
    import "../app.css";
    import Header from "$lib/components/Header.svelte";
    import { onMount } from "svelte";
    import { registerSW } from "virtual:pwa-register";
    import { pwa } from "$lib/state/pwa.svelte.js";

    let { children } = $props();

    onMount(() => {
        registerSW();

        window.addEventListener("beforeinstallprompt", (e) => {
            e.preventDefault();
            pwa.deferredPrompt = e;
        });

        window.addEventListener("appinstalled", () => {
            pwa.deferredPrompt = null;
        });
    });
</script>

<div class="min-h-screen bg-gray-50 flex flex-col">
    <Header />
    <main class="flex-1 max-w-5xl mx-auto w-full px-4 py-8">
        {@render children()}
    </main>
</div>

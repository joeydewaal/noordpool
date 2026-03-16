<script lang="ts">
    import "../app.css";
    import Header from "$lib/components/Header.svelte";
    import { onMount } from "svelte";
    import { registerSW } from "virtual:pwa-register";
    import { pwa } from "$lib/state/pwa.svelte.js";
    import { theme } from "$lib/state/theme.svelte.js";

    let { children } = $props();

    onMount(() => {
        theme.init();
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

<div class="min-h-screen flex">
    <Header />
    <main class="flex-1 max-w-5xl w-full px-4 py-8 md:px-8 pb-24 md:pb-8">
        {@render children()}
    </main>
</div>

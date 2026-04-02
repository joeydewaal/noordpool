<script lang="ts">
    import "../app.css";
    import Header from "$lib/components/Header.svelte";
    import { onMount } from "svelte";
    import { registerSW } from "virtual:pwa-register";
    import { pwa } from "$lib/state/pwa.svelte";
    import { theme } from "$lib/state/theme.svelte";
    import { QueryClient } from '@tanstack/svelte-query';
    import { PersistQueryClientProvider } from '@tanstack/svelte-query-persist-client';
    import { get, set, del } from 'idb-keyval';
    import type { Persister } from '@tanstack/query-persist-client-core';

    let { children } = $props();

    const queryClient = new QueryClient({
        defaultOptions: {
            queries: {
                gcTime: 1000 * 60 * 60 * 24, // 24 hours
            },
        },
    });

    const persister: Persister = {
        persistClient: (client) => set('noordpool-query-cache', client),
        restoreClient: () => get('noordpool-query-cache'),
        removeClient: () => del('noordpool-query-cache'),
    };

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

<PersistQueryClientProvider client={queryClient} persistOptions={{ persister }}>
    <div class="min-h-screen flex">
        <Header />
        <main class="flex-1 max-w-5xl w-full px-4 py-8 md:px-8 pb-24 md:pb-8">
            {@render children()}
        </main>
    </div>
</PersistQueryClientProvider>

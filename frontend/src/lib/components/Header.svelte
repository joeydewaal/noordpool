<script lang="ts">
    import { auth } from "$lib/state/auth.svelte";
    import { pwa } from "$lib/state/pwa.svelte";
    import { theme } from "$lib/state/theme.svelte";
    import { Navigation } from "@skeletonlabs/skeleton-svelte";
    import { createQuery } from "@tanstack/svelte-query";
    import { browser } from "$app/environment";
    import { getUpcomingGames } from "$lib/api/games";
    import type { Game } from "$lib/api/types";

    // Used to flag a game-day in the navigation. Shares its query key
    // with the /games page so TanStack Query dedupes the request.
    // Polled every 60 s while the tab is visible — covers status
    // transitions from `scheduled` to `live` to `finished` without
    // SSE/WebSockets (Lambda deploy target — see project memory).
    const upcomingQuery = createQuery(() => ({
        queryKey: ["games", "upcoming"],
        queryFn: () => getUpcomingGames(),
        refetchInterval: 60_000,
        refetchIntervalInBackground: false,
        staleTime: 30_000,
        enabled: browser,
    }));

    const todaysGame = $derived.by((): Game | null => {
        const list = (upcomingQuery.data ?? []) as Game[];
        if (list.length === 0) return null;
        const now = new Date();
        const sameDay = (g: Game) => {
            const dt = new Date(g.dateTime);
            return (
                dt.getFullYear() === now.getFullYear() &&
                dt.getMonth() === now.getMonth() &&
                dt.getDate() === now.getDate()
            );
        };
        const today = list
            .filter(sameDay)
            .slice()
            .sort(
                (a, b) =>
                    new Date(a.dateTime).getTime() -
                    new Date(b.dateTime).getTime(),
            );
        return today.find((g) => g.status === "live") ?? today[0] ?? null;
    });
    const isLiveToday = $derived(todaysGame?.status === "live");
</script>

<!-- Desktop sidebar -->
<div class="hidden md:flex h-screen sticky top-0">
    <Navigation
        layout="sidebar"
        class="h-full flex flex-col border-r border-surface-200 dark:border-surface-700"
    >
        <Navigation.Header class="flex items-center gap-3 mb-4">
            <a href="/" class="flex items-center gap-3">
                <img
                    src={theme.isDark ? "/icons/white.jpg" : "/icons/black.jpg"}
                    alt="Noordpool"
                    class="w-10 h-10 rounded"
                />
                <span class="text-xl font-bold tracking-tight">Noordpool</span>
            </a>
        </Navigation.Header>
        <Navigation.Menu>
            <Navigation.Group>
                <Navigation.TriggerAnchor href="/">
                    <svg
                        class="w-5 h-5"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        viewBox="0 0 24 24"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-4 0a1 1 0 01-1-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 01-1 1"
                        />
                    </svg>
                    <Navigation.TriggerText>Home</Navigation.TriggerText>
                </Navigation.TriggerAnchor>
                <Navigation.TriggerAnchor href="/games">
                    <span class="relative inline-block">
                        <svg
                            class="w-5 h-5"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                            viewBox="0 0 24 24"
                        >
                            <path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"
                            />
                        </svg>
                        {#if todaysGame}
                            <span
                                class="absolute -top-0.5 -right-0.5 w-2 h-2 rounded-full bg-error-500 {isLiveToday
                                    ? 'animate-pulse'
                                    : ''}"
                                aria-label={isLiveToday
                                    ? "Wedstrijd live"
                                    : "Wedstrijd vandaag"}
                            ></span>
                        {/if}
                    </span>
                    <Navigation.TriggerText>
                        <span class="flex flex-col leading-tight">
                            <span>Wedstrijden</span>
                            {#if todaysGame}
                                <span class="text-xs text-surface-400">
                                    vs {todaysGame.opponent}
                                </span>
                            {/if}
                        </span>
                    </Navigation.TriggerText>
                </Navigation.TriggerAnchor>
                <Navigation.TriggerAnchor href="/players">
                    <svg
                        class="w-5 h-5"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        viewBox="0 0 24 24"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0z"
                        />
                    </svg>
                    <Navigation.TriggerText>Spelers</Navigation.TriggerText>
                </Navigation.TriggerAnchor>
                <Navigation.TriggerAnchor href="/stats">
                    <svg
                        class="w-5 h-5"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        viewBox="0 0 24 24"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"
                        />
                    </svg>
                    <Navigation.TriggerText>Statistieken</Navigation.TriggerText
                    >
                </Navigation.TriggerAnchor>
            </Navigation.Group>
        </Navigation.Menu>
        <Navigation.Footer class="mt-auto flex flex-col gap-2">
            {#if pwa.installable}
                <Navigation.Trigger onclick={() => pwa.install()}>
                    <svg
                        class="w-5 h-5"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        viewBox="0 0 24 24"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="M4 16v2a2 2 0 002 2h12a2 2 0 002-2v-2M7 10l5 5 5-5M12 15V3"
                        />
                    </svg>
                    <Navigation.TriggerText>Installeer</Navigation.TriggerText>
                </Navigation.Trigger>
            {/if}
            {#if auth.isAuthenticated}
                <div class="text-sm text-surface-400 px-2">
                    {auth.user?.firstName}
                    {auth.user?.lastName}
                </div>
                <Navigation.TriggerAnchor href="/profile">
                    <svg
                        class="w-5 h-5"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        viewBox="0 0 24 24"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"
                        />
                    </svg>
                    <Navigation.TriggerText>Profiel</Navigation.TriggerText>
                </Navigation.TriggerAnchor>
            {:else}
                <Navigation.TriggerAnchor href="/auth/login">
                    <svg
                        class="w-5 h-5"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        viewBox="0 0 24 24"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="M11 16l-4-4m0 0l4-4m-4 4h14m-5 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"
                        />
                    </svg>
                    <Navigation.TriggerText>Inloggen</Navigation.TriggerText>
                </Navigation.TriggerAnchor>
            {/if}
        </Navigation.Footer>
    </Navigation>
</div>

<!-- Mobile bottom bar -->
<div
    class="md:hidden fixed bottom-0 left-0 right-0 z-50 border-t border-surface-200 dark:border-surface-700"
>
    <Navigation layout="bar">
        <Navigation.Menu>
            <Navigation.TriggerAnchor href="/">
                <svg
                    class="w-5 h-5"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    viewBox="0 0 24 24"
                >
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-4 0a1 1 0 01-1-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 01-1 1"
                    />
                </svg>
                <Navigation.TriggerText>Home</Navigation.TriggerText>
            </Navigation.TriggerAnchor>
            <Navigation.TriggerAnchor href="/games">
                <span class="relative inline-block">
                    <svg
                        class="w-5 h-5"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        viewBox="0 0 24 24"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"
                        />
                    </svg>
                    {#if todaysGame}
                        <span
                            class="absolute -top-0.5 -right-0.5 w-2 h-2 rounded-full bg-error-500 {isLiveToday
                                ? 'animate-pulse'
                                : ''}"
                            aria-label={isLiveToday
                                ? "Wedstrijd live"
                                : "Wedstrijd vandaag"}
                        ></span>
                    {/if}
                </span>
                <Navigation.TriggerText>Wedstrijden</Navigation.TriggerText>
            </Navigation.TriggerAnchor>
            <Navigation.TriggerAnchor href="/players">
                <svg
                    class="w-5 h-5"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    viewBox="0 0 24 24"
                >
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        d="M17 20h5v-2a3 3 0 00-5.356-1.857M17 20H7m10 0v-2c0-.656-.126-1.283-.356-1.857M7 20H2v-2a3 3 0 015.356-1.857M7 20v-2c0-.656.126-1.283.356-1.857m0 0a5.002 5.002 0 019.288 0M15 7a3 3 0 11-6 0 3 3 0 016 0z"
                    />
                </svg>
                <Navigation.TriggerText>Spelers</Navigation.TriggerText>
            </Navigation.TriggerAnchor>
            <Navigation.TriggerAnchor href="/stats">
                <svg
                    class="w-5 h-5"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    viewBox="0 0 24 24"
                >
                    <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"
                    />
                </svg>
                <Navigation.TriggerText>Stats</Navigation.TriggerText>
            </Navigation.TriggerAnchor>
            {#if auth.isAuthenticated}
                <Navigation.TriggerAnchor href="/profile">
                    <svg
                        class="w-5 h-5"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        viewBox="0 0 24 24"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z"
                        />
                    </svg>
                    <Navigation.TriggerText>Profiel</Navigation.TriggerText>
                </Navigation.TriggerAnchor>
            {:else}
                <Navigation.TriggerAnchor href="/auth/login">
                    <svg
                        class="w-5 h-5"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="2"
                        viewBox="0 0 24 24"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="M11 16l-4-4m0 0l4-4m-4 4h14m-5 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"
                        />
                    </svg>
                    <Navigation.TriggerText>Login</Navigation.TriggerText>
                </Navigation.TriggerAnchor>
            {/if}
        </Navigation.Menu>
    </Navigation>
</div>

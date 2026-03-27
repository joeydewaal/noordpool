<script lang="ts">
    import { createQuery } from '@tanstack/svelte-query';
    import { page } from "$app/state";
    import { replaceState } from "$app/navigation";
    import { auth } from "$lib/state/auth.svelte.ts";
    import { getUpcomingGames, getRecentResults } from "$lib/api/games.ts";
    import type { Game } from "$lib/api/types.ts";
    import { Tabs } from "@skeletonlabs/skeleton-svelte";

    const canManage = $derived(auth.isAdmin || auth.isModerator);

    const upcomingQuery = createQuery(() => ({
        queryKey: ['games', 'upcoming'],
        queryFn: () => getUpcomingGames(),
    }));

    const recentQuery = createQuery(() => ({
        queryKey: ['games', 'recent'],
        queryFn: () => getRecentResults(),
    }));

    function formatDate(dateTime: string): string {
        return new Date(dateTime).toLocaleDateString("nl-NL", {
            weekday: "short",
            day: "numeric",
            month: "short",
            year: "numeric",
            hour: "2-digit",
            minute: "2-digit",
        });
    }

    let activeTab = $state(page.url.searchParams.get('tab') ?? 'upcoming');

    $effect(() => {
        activeTab = page.url.searchParams.get('tab') ?? 'upcoming';
    });

    function onTabChange(details: { value: string }) {
        activeTab = details.value;
        const url = new URL(page.url);
        url.searchParams.set('tab', details.value);
        replaceState(url, {});
    }

    function formatScore(game: Game): string {
        if (
            game.homeScore === null ||
            (game.status === "scheduled" && game.homeScore === 0)
        )
            return "";
        return `${game.homeScore} - ${game.awayScore}`;
    }
</script>

<div class="flex items-center justify-between mb-6">
    <h1 class="text-2xl font-bold">Wedstrijden</h1>
    {#if canManage}
        <a href="/games/new" class="btn btn-sm preset-filled-primary-500">
            Nieuwe wedstrijd
        </a>
    {/if}
</div>

{#if upcomingQuery.isError || recentQuery.isError}
    <p class="text-red-500 text-sm">Kon wedstrijden niet laden</p>
{:else}
    <Tabs value={activeTab} onValueChange={onTabChange}>
        <Tabs.List class="mb-6">
            <Tabs.Trigger value="upcoming">Komend</Tabs.Trigger>
            <Tabs.Trigger value="results">Uitslagen</Tabs.Trigger>
        </Tabs.List>
        <Tabs.Content value="upcoming">
            {#if upcomingQuery.isPending}
                <p class="text-surface-400 text-sm">Laden...</p>
            {:else if !upcomingQuery.data || upcomingQuery.data.length === 0}
                <p class="text-surface-400 text-sm">
                    Geen komende wedstrijden gepland.
                </p>
            {:else}
                <div class="space-y-3">
                    {#each upcomingQuery.data as game}
                        <a
                            href="/games/{game.id}"
                            class="block card preset-tonal-surface p-4 hover:preset-tonal-primary transition-colors"
                        >
                            <div class="flex items-center justify-between">
                                <div>
                                    <div class="font-semibold">
                                        vs {game.opponent}
                                    </div>
                                    <div class="text-sm text-surface-400 mt-1">
                                        {formatDate(game.dateTime)}
                                    </div>
                                </div>
                                <span
                                    class="chip {game.homeAway === 'home'
                                        ? 'preset-filled-success-500'
                                        : 'preset-filled-secondary-500'}"
                                >
                                    {game.homeAway === "home" ? "thuis" : "uit"}
                                </span>
                            </div>
                        </a>
                    {/each}
                </div>
            {/if}
        </Tabs.Content>
        <Tabs.Content value="results">
            {#if recentQuery.isPending}
                <p class="text-surface-400 text-sm">Laden...</p>
            {:else if !recentQuery.data || recentQuery.data.length === 0}
                <p class="text-surface-400 text-sm">Nog geen uitslagen.</p>
            {:else}
                <div class="space-y-3">
                    {#each recentQuery.data as game}
                        <a
                            href="/games/{game.id}"
                            class="block card preset-tonal-surface p-4 hover:preset-tonal-primary transition-colors"
                        >
                            <div class="flex items-center justify-between">
                                <div>
                                    <div class="font-semibold">
                                        vs {game.opponent}
                                    </div>
                                    <div class="text-sm text-surface-400 mt-1">
                                        {formatDate(game.dateTime)}
                                    </div>
                                </div>
                                <div class="flex items-center gap-3">
                                    <span class="font-bold"
                                        >{formatScore(game)}</span
                                    >
                                    <span
                                        class="chip {game.homeAway === 'home'
                                            ? 'preset-filled-success-500'
                                            : 'preset-filled-secondary-500'}"
                                    >
                                        {game.homeAway === "home"
                                            ? "thuis"
                                            : "uit"}
                                    </span>
                                </div>
                            </div>
                        </a>
                    {/each}
                </div>
            {/if}
        </Tabs.Content>
    </Tabs>
{/if}

<script lang="ts">
    import { createQuery } from "@tanstack/svelte-query";
    import { auth } from "$lib/state/auth.svelte";
    import { getGamesSummary } from "$lib/api/games";
    import type { Game } from "$lib/api/types";

    const summaryQuery = createQuery(() => ({
        queryKey: ["games", "summary"],
        queryFn: () => getGamesSummary(3),
    }));

    function formatDate(dateTime: string): string {
        return new Date(dateTime).toLocaleDateString("nl-NL", {
            weekday: "short",
            day: "numeric",
            month: "short",
            hour: "2-digit",
            minute: "2-digit",
        });
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

{#if auth.isAuthenticated}
    <div>
        <h1 class="text-2xl font-bold mb-4">
            Welkom terug, {auth.user?.firstName} {auth.user?.lastName}!
        </h1>
        <div class="grid gap-6 md:grid-cols-2">
            <div class="card p-6">
                <div class="flex items-center justify-between mb-3">
                    <h2 class="text-lg font-semibold">Komende wedstrijden</h2>
                    <a
                        href="/games"
                        class="text-sm text-primary-500 hover:underline"
                        >Bekijk alles</a
                    >
                </div>
                {#if !summaryQuery.data?.upcoming || summaryQuery.data.upcoming.length === 0}
                    <p class="text-surface-400 text-sm">
                        Geen komende wedstrijden.
                    </p>
                {:else}
                    <div class="space-y-3">
                        {#each summaryQuery.data.upcoming as game}
                            <a
                                href="/games/{game.id}"
                                class="block p-3 rounded-lg hover:preset-tonal-surface transition-colors -mx-1"
                            >
                                <div class="flex items-center justify-between">
                                    <div class="font-medium text-sm">
                                        vs {game.opponent}
                                    </div>
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
                                <div class="text-xs text-surface-400 mt-1">
                                    {formatDate(game.dateTime)}
                                </div>
                            </a>
                        {/each}
                    </div>
                {/if}
            </div>
            <div class="card p-6">
                <div class="flex items-center justify-between mb-3">
                    <h2 class="text-lg font-semibold">Recente uitslagen</h2>
                    <a
                        href="/games"
                        class="text-sm text-primary-500 hover:underline"
                        >Bekijk alles</a
                    >
                </div>
                {#if !summaryQuery.data?.recent || summaryQuery.data.recent.length === 0}
                    <p class="text-surface-400 text-sm">Nog geen uitslagen.</p>
                {:else}
                    <div class="space-y-3">
                        {#each summaryQuery.data.recent as game}
                            <a
                                href="/games/{game.id}"
                                class="block p-3 rounded-lg hover:preset-tonal-surface transition-colors -mx-1"
                            >
                                <div class="flex items-center justify-between">
                                    <div class="font-medium text-sm">
                                        vs {game.opponent}
                                    </div>
                                    <div class="flex items-center gap-2">
                                        <span class="font-bold text-sm"
                                            >{formatScore(game)}</span
                                        >
                                        <span
                                            class="chip {game.homeAway ===
                                            'home'
                                                ? 'preset-filled-success-500'
                                                : 'preset-filled-secondary-500'}"
                                        >
                                            {game.homeAway === "home"
                                                ? "thuis"
                                                : "uit"}
                                        </span>
                                    </div>
                                </div>
                                <div class="text-xs text-surface-400 mt-1">
                                    {formatDate(game.dateTime)}
                                </div>
                            </a>
                        {/each}
                    </div>
                {/if}
            </div>
        </div>
    </div>
{:else}
    <div class="text-center py-16">
        <h1 class="text-4xl font-bold mb-4">Noordpool</h1>
        <p class="text-lg text-surface-400 mb-8">
            Welkom bij de Noordpool voetbal app.
        </p>
        <a href="/auth/login" class="btn preset-filled-primary-500">
            Aan de slag
        </a>
    </div>
    <div class="mt-12 grid gap-6 md:grid-cols-2">
        <div class="card p-6">
            <div class="flex items-center justify-between mb-3">
                <h2 class="text-lg font-semibold">Komende wedstrijden</h2>
                <a
                    href="/games"
                    class="text-sm text-primary-500 hover:underline"
                    >Bekijk alles</a
                >
            </div>
            {#if !summaryQuery.data?.upcoming || summaryQuery.data.upcoming.length === 0}
                <p class="text-surface-400 text-sm">
                    Geen komende wedstrijden.
                </p>
            {:else}
                <div class="space-y-3">
                    {#each summaryQuery.data.upcoming as game}
                        <a
                            href="/games/{game.id}"
                            class="block p-3 rounded-lg hover:preset-tonal-surface transition-colors -mx-1"
                        >
                            <div class="flex items-center justify-between">
                                <div class="font-medium text-sm">
                                    vs {game.opponent}
                                </div>
                                <span
                                    class="chip {game.homeAway === 'home'
                                        ? 'preset-filled-success-500'
                                        : 'preset-filled-secondary-500'}"
                                >
                                    {game.homeAway === "home" ? "thuis" : "uit"}
                                </span>
                            </div>
                            <div class="text-xs text-surface-400 mt-1">
                                {formatDate(game.dateTime)}
                            </div>
                        </a>
                    {/each}
                </div>
            {/if}
        </div>
        <div class="card p-6">
            <div class="flex items-center justify-between mb-3">
                <h2 class="text-lg font-semibold">Recente uitslagen</h2>
                <a
                    href="/games"
                    class="text-sm text-primary-500 hover:underline"
                    >Bekijk alles</a
                >
            </div>
            {#if !summaryQuery.data?.recent || summaryQuery.data.recent.length === 0}
                <p class="text-surface-400 text-sm">Nog geen uitslagen.</p>
            {:else}
                <div class="space-y-3">
                    {#each summaryQuery.data.recent as game}
                        <a
                            href="/games/{game.id}"
                            class="block p-3 rounded-lg hover:preset-tonal-surface transition-colors -mx-1"
                        >
                            <div class="flex items-center justify-between">
                                <div class="font-medium text-sm">
                                    vs {game.opponent}
                                </div>
                                <div class="flex items-center gap-2">
                                    <span class="font-bold text-sm"
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
                            <div class="text-xs text-surface-400 mt-1">
                                {formatDate(game.dateTime)}
                            </div>
                        </a>
                    {/each}
                </div>
            {/if}
        </div>
    </div>
{/if}

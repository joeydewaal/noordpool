<script lang="ts">
  import { createQuery } from "@tanstack/svelte-query";
  import { page } from "$app/state";
  import { replaceState } from "$app/navigation";
  import { auth } from "$lib/state/auth.svelte";
  import { getUpcomingGames, getRecentResults } from "$lib/api/games";
  import type { Game, ScoreSide } from "$lib/api/types";
  import { Tabs } from "@skeletonlabs/skeleton-svelte";
  import { isThisWeek, isToday } from "$lib/utils/date";

  const canManage = $derived(auth.isAdmin || auth.isModerator);

  const upcomingQuery = createQuery(() => ({
    queryKey: ["games", "upcoming"],
    queryFn: () => getUpcomingGames(),
  }));

  const recentQuery = createQuery(() => ({
    queryKey: ["games", "recent"],
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

  let activeTab = $state(page.url.searchParams.get("tab") ?? "upcoming");

  $effect(() => {
    activeTab = page.url.searchParams.get("tab") ?? "upcoming";
  });

  function onTabChange(details: { value: string }) {
    activeTab = details.value;
    const url = new URL(page.url);
    url.searchParams.set("tab", details.value);
    replaceState(url, {});
  }

  function formatScore(game: Game): string {
    return `${game.homeScore} - ${game.awayScore}`;
  }

  function ownSide(game: Game): ScoreSide | null {
    const teamId = auth.teamId;
    if (!teamId) return null;
    if (game.homeTeam.id === teamId) return "home";
    if (game.awayTeam.id === teamId) return "away";
    return null;
  }

  const thisWeekMatch: Game | null = $derived.by(() => {
    const games = upcomingQuery.data;
    if (!games || games.length === 0) return null;
    const next = games[0];
    return isThisWeek(next.dateTime) ? next : null;
  });

  const remainingUpcoming: Game[] = $derived.by(() => {
    const games = upcomingQuery.data;
    if (!games) return [];
    if (!thisWeekMatch) return games;
    return games.filter((g) => g.id !== thisWeekMatch.id);
  });
</script>

<div class="flex items-center justify-between mb-6">
  <h1 class="text-2xl font-bold">Wedstrijden</h1>
  {#if canManage}
    <a href="/games/new" class="btn btn-sm preset-filled-primary-500">
      Nieuwe wedstrijd
    </a>
  {/if}
</div>

{#if thisWeekMatch}
  {@const side = ownSide(thisWeekMatch)}
  <a
    href="/games/{thisWeekMatch.id}"
    class="block card p-5 mb-6 border-l-4 border-primary-500 preset-tonal-primary hover:brightness-125 transition-all"
  >
    <div class="flex items-center justify-between mb-2">
      <span class="chip preset-filled-primary-500 text-xs font-semibold">
        {#if thisWeekMatch.status === "live"}
          <span
            class="inline-block w-2 h-2 rounded-full bg-white mr-1 animate-pulse"
          ></span>
          LIVE
        {:else if isToday(thisWeekMatch.dateTime)}
          Vandaag
        {:else}
          Deze week
        {/if}
      </span>
      {#if side}
        <span
          class="chip {side === 'home'
            ? 'preset-filled-success-500'
            : 'preset-filled-secondary-500'}"
        >
          {side === "home" ? "thuis" : "uit"}
        </span>
      {/if}
    </div>
    <div class="text-lg font-bold">
      {thisWeekMatch.homeTeam.name} vs {thisWeekMatch.awayTeam.name}
    </div>
    <div class="text-sm text-surface-400 mt-1">
      {formatDate(thisWeekMatch.dateTime)}
    </div>
    <div class="text-sm text-surface-400">{thisWeekMatch.location}</div>
    {#if thisWeekMatch.status === "live"}
      <div class="text-xl font-bold mt-2">
        {thisWeekMatch.homeScore} – {thisWeekMatch.awayScore}
      </div>
    {/if}
  </a>
{/if}

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
      {:else if remainingUpcoming.length === 0 && !thisWeekMatch}
        <p class="text-surface-400 text-sm">
          Geen komende wedstrijden gepland.
        </p>
      {:else if remainingUpcoming.length === 0}
        <p class="text-surface-400 text-sm">
          Geen andere komende wedstrijden gepland.
        </p>
      {:else}
        <div class="space-y-3">
          {#each remainingUpcoming as game}
            {@const side = ownSide(game)}
            <a
              href="/games/{game.id}"
              class="block card preset-tonal-surface p-4 hover:preset-tonal-primary transition-colors"
            >
              <div class="flex items-center justify-between">
                <div>
                  <div class="font-semibold">
                    {game.homeTeam.name} vs {game.awayTeam.name}
                  </div>
                  <div class="text-sm text-surface-400 mt-1">
                    {formatDate(game.dateTime)}
                  </div>
                </div>
                <div class="flex items-center gap-2">
                  {#if game.status === "live"}
                    <span
                      class="chip preset-filled-error-500 animate-pulse"
                      title="Wedstrijd is bezig"
                    >
                      <span
                        class="inline-block w-2 h-2 rounded-full bg-white mr-1"
                      ></span>
                      LIVE {game.homeScore}–{game.awayScore}
                    </span>
                  {/if}
                  {#if side}
                    <span
                      class="chip {side === 'home'
                        ? 'preset-filled-success-500'
                        : 'preset-filled-secondary-500'}"
                    >
                      {side === "home" ? "thuis" : "uit"}
                    </span>
                  {/if}
                </div>
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
            {@const side = ownSide(game)}
            <a
              href="/games/{game.id}"
              class="block card preset-tonal-surface p-4 hover:preset-tonal-primary transition-colors"
            >
              <div class="flex items-center justify-between">
                <div>
                  <div class="font-semibold">
                    {game.homeTeam.name} vs {game.awayTeam.name}
                  </div>
                  <div class="text-sm text-surface-400 mt-1">
                    {formatDate(game.dateTime)}
                  </div>
                </div>
                <div class="flex items-center gap-3">
                  <span class="font-bold">{formatScore(game)}</span>
                  {#if side}
                    <span
                      class="chip {side === 'home'
                        ? 'preset-filled-success-500'
                        : 'preset-filled-secondary-500'}"
                    >
                      {side === "home" ? "thuis" : "uit"}
                    </span>
                  {/if}
                </div>
              </div>
            </a>
          {/each}
        </div>
      {/if}
    </Tabs.Content>
  </Tabs>
{/if}

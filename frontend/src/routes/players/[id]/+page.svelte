<script lang="ts">
  import { page } from "$app/state";
  import { auth } from "$lib/state/auth.svelte";
  import { getPlayer, updatePlayer } from "$lib/api/players";
  import { getPlayerStats } from "$lib/api/events";
  import {
    createQuery,
    createMutation,
    useQueryClient,
  } from "@tanstack/svelte-query";
  import type { UpdatePlayerRequest } from "$lib/api/types";
  import PlayerStatsBar from "$lib/components/charts/PlayerStatsBar.svelte";
  import PlayerTimeline from "$lib/components/charts/PlayerTimeline.svelte";

  const id = page.params.id!;
  const queryClient = useQueryClient();

  const canManage = $derived(auth.isAdmin || auth.isModerator);

  const playerQuery = createQuery(() => ({
    queryKey: ["players", id],
    queryFn: () => getPlayer(id),
  }));

  const statsQuery = createQuery(() => ({
    queryKey: ["players", id, "stats"],
    queryFn: () => getPlayerStats(id),
  }));

  const toggleActiveMutation = createMutation(() => ({
    mutationFn: (data: UpdatePlayerRequest) => updatePlayer(id, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["players", id] });
    },
  }));

  const positionColor: Record<string, string> = {
    Keeper: "preset-filled-warning-500",
    "Centrale verdediger": "preset-filled-secondary-500",
    Linksback: "preset-filled-secondary-500",
    Rechtsback: "preset-filled-secondary-500",
    "Defensieve middenvelder": "preset-filled-primary-500",
    "Centrale middenvelder": "preset-filled-primary-500",
    "Aanvallende middenvelder": "preset-filled-primary-500",
    Linksvleugel: "preset-filled-error-500",
    Rechtsvleugel: "preset-filled-error-500",
    Spits: "preset-filled-error-500",
  };

  function toggleActive() {
    const player = playerQuery.data;
    if (!player) return;
    toggleActiveMutation.mutate({ active: !player.active });
  }
</script>

{#if playerQuery.data}
  <div class="max-w-2xl">
    <button
      onclick={() => history.back()}
      class="text-sm text-primary-500 hover:underline mb-4 inline-block"
      >&larr; Terug</button
    >
    <div class="card p-6">
      <div class="flex items-center gap-4 mb-4">
        <div class="text-4xl font-bold text-primary-500">
          {playerQuery.data.shirtNumber}
        </div>
        <div>
          <h1 class="text-2xl font-bold">
            {playerQuery.data.firstName}
            {playerQuery.data.lastName}
          </h1>
          <span class="chip mt-1 {positionColor[playerQuery.data.position]}">
            {playerQuery.data.position}
          </span>
          {#if !playerQuery.data.active}
            <span class="chip mt-1 ml-1 preset-tonal-surface"> inactief </span>
          {/if}
          {#if playerQuery.data.user?.isAdmin}
            <span class="chip mt-1 ml-1 preset-filled-warning-500">
              admin
            </span>
          {:else if playerQuery.data.user?.isModerator}
            <span class="chip mt-1 ml-1 preset-filled-tertiary-500">
              moderator
            </span>
          {/if}
        </div>
      </div>

      {#if statsQuery.data}
        <div class="grid grid-cols-5 gap-2 mt-4">
          {#each [{ value: statsQuery.data.appearances, label: "Wedstrijden" }, { value: statsQuery.data.goals, label: "Doelpunten" }, { value: statsQuery.data.assists, label: "Assists" }, { value: statsQuery.data.yellowCards, label: "Gele kaarten" }, { value: statsQuery.data.redCards, label: "Rode kaarten" }] as item}
            <div class="text-center p-2 card preset-tonal-surface rounded-lg">
              <div class="text-2xl font-bold">{item.value}</div>
              <div class="text-xs text-surface-400 mt-1">
                {item.label}
              </div>
            </div>
          {/each}
        </div>
      {/if}

      {#if statsQuery.data}
        <h2 class="text-lg font-bold mt-8 mb-4">Statistieken</h2>

        <div class="card preset-tonal-surface p-4 mb-4">
          <h3 class="text-sm font-medium text-surface-400 mb-2">Overzicht</h3>
          <PlayerStatsBar
            goals={statsQuery.data.goals}
            assists={statsQuery.data.assists}
            yellowCards={statsQuery.data.yellowCards}
            redCards={statsQuery.data.redCards}
          />
        </div>

        <div class="card preset-tonal-surface p-4">
          <h3 class="text-sm font-medium text-surface-400 mb-2">
            Seizoensverloop
          </h3>
          <PlayerTimeline timeline={statsQuery.data.gameTimeline} />
        </div>
      {/if}

      {#if canManage}
        <div
          class="flex gap-3 mt-6 pt-4 border-t border-surface-200 dark:border-surface-800"
        >
          <a
            href="/players/{playerQuery.data.id}/edit"
            class="btn btn-sm preset-filled-primary-500"
          >
            Bewerken
          </a>
          <button
            onclick={toggleActive}
            class="btn btn-sm preset-outlined-surface-500"
          >
            {playerQuery.data.active ? "Deactiveren" : "Activeren"}
          </button>
        </div>
      {/if}
    </div>
  </div>
{:else if playerQuery.isError}
  <p class="text-surface-400">Speler niet gevonden.</p>
{/if}

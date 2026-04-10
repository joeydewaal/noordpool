<script lang="ts">
  import { createQuery } from "@tanstack/svelte-query";
  import { auth } from "$lib/state/auth.svelte";
  import { getPlayers } from "$lib/api/players";

  const canManage = $derived(auth.isAdmin || auth.isModerator);

  let showInactive = $state(false);

  const playersQuery = createQuery(() => ({
    queryKey: ["players"],
    queryFn: getPlayers,
  }));

  const filtered = $derived(
    showInactive
      ? (playersQuery.data ?? [])
      : (playersQuery.data ?? []).filter((p) => p.active),
  );

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
</script>

<div class="flex items-center justify-between mb-6">
  <h1 class="text-2xl font-bold">Spelers</h1>
  <div class="flex items-center gap-3">
    {#if canManage}
      <label class="flex items-center gap-2 text-sm text-surface-400">
        <input type="checkbox" bind:checked={showInactive} class="checkbox" />
        Toon inactief
      </label>
      <a href="/players/new" class="btn btn-sm preset-filled-primary-500">
        Speler toevoegen
      </a>
    {/if}
  </div>
</div>

<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
  {#each filtered as player}
    <a
      href="/players/{player.id}"
      class="card preset-tonal-surface p-5 flex items-center gap-4 hover:preset-tonal-primary transition-colors {!player.active
        ? 'opacity-60'
        : ''}"
    >
      <div class="text-2xl font-bold text-primary-500 w-12 text-center">
        {player.shirtNumber}
      </div>
      <div class="flex-1 min-w-0">
        <div class="font-semibold truncate">
          {player.firstName}
          {player.lastName}
        </div>
        <span class="chip mt-1 {positionColor[player.position]}">
          {player.position}
        </span>
        {#if !player.active}
          <span class="chip mt-1 ml-1 preset-tonal-surface"> inactief </span>
        {/if}
      </div>
    </a>
  {/each}
</div>

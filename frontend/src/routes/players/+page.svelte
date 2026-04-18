<script lang="ts">
  import { createQuery } from "@tanstack/svelte-query";
  import { auth } from "$lib/state/auth.svelte";
  import { getPlayers } from "$lib/api/players";
  import Spinner from "$lib/components/Spinner.svelte";
  import PlayerAvatar from "$lib/components/PlayerAvatar.svelte";

  const canManage = $derived(auth.isAdmin || auth.isModerator);

  let showInactive = $state(false);

  const playersQuery = createQuery(() => ({
    queryKey: ["players"],
    queryFn: getPlayers,
  }));

  const POSITION_ORDER: Record<string, number> = {
    Spits: 1,
    Linksvleugel: 2,
    Rechtsvleugel: 3,
    "Aanvallende middenvelder": 4,
    "Centrale middenvelder": 5,
    "Defensieve middenvelder": 6,
    Linksback: 7,
    Rechtsback: 8,
    "Centrale verdediger": 9,
    Keeper: 10,
  };

  const POSITION_GROUP: Record<string, string> = {
    Spits: "Aanvallers",
    Linksvleugel: "Aanvallers",
    Rechtsvleugel: "Aanvallers",
    "Aanvallende middenvelder": "Middenvelders",
    "Centrale middenvelder": "Middenvelders",
    "Defensieve middenvelder": "Middenvelders",
    Linksback: "Verdedigers",
    Rechtsback: "Verdedigers",
    "Centrale verdediger": "Verdedigers",
    Keeper: "Keeper",
  };

  const sorted = $derived(
    [
      ...(showInactive
        ? (playersQuery.data ?? [])
        : (playersQuery.data ?? []).filter((p) => p.active)),
    ].sort((a, b) => {
      const orderDiff =
        (POSITION_ORDER[a.position] ?? 99) - (POSITION_ORDER[b.position] ?? 99);
      if (orderDiff !== 0) return orderDiff;
      return a.lastName.localeCompare(b.lastName, "nl");
    }),
  );

  type Player = NonNullable<typeof playersQuery.data>[number];
  const grouped = $derived(
    sorted.reduce<{ group: string; players: Player[] }[]>((acc, player) => {
      const group = POSITION_GROUP[player.position] ?? "Overig";
      const last = acc[acc.length - 1];
      if (last?.group === group) {
        last.players.push(player);
      } else {
        acc.push({ group, players: [player] });
      }
      return acc;
    }, []),
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

{#if playersQuery.isPending}
  <Spinner />
{:else if playersQuery.isError}
  <p class="text-error-500 text-sm">Kon spelers niet laden</p>
{:else}
  <div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
    {#each grouped as { group, players }}
      <h2
        class="col-span-full text-sm font-semibold text-surface-400 uppercase tracking-wide mt-4 first:mt-0"
      >
        {group}
      </h2>
      {#each players as player}
        <a
          href="/players/{player.id}"
          class="card preset-tonal-surface p-5 flex items-center gap-4 hover:preset-tonal-primary transition-colors {!player.active
            ? 'opacity-60'
            : ''}"
        >
          <PlayerAvatar
            avatarUrl={player.user?.avatarUrl}
            shirtNumber={player.shirtNumber}
            name="{player.firstName} {player.lastName}"
          />
          <div class="flex-1 min-w-0">
            <div class="font-semibold truncate">
              {player.firstName}
              {player.lastName}
            </div>
            <span class="chip mt-1 {positionColor[player.position]}">
              {player.position}
            </span>
            {#if !player.active}
              <span class="chip mt-1 ml-1 preset-tonal-surface">
                inactief
              </span>
            {/if}
          </div>
        </a>
      {/each}
    {/each}
  </div>
{/if}

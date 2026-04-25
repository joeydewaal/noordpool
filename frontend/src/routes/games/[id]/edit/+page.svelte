<script lang="ts">
  import { page } from "$app/state";
  import { goto } from "$app/navigation";
  import { auth } from "$lib/state/auth.svelte";
  import { getGame, updateGame } from "$lib/api/games";
  import { listTeams } from "$lib/api/teams";
  import {
    createQuery,
    createMutation,
    useQueryClient,
  } from "@tanstack/svelte-query";
  import type { Game, Team, UpdateGameRequest } from "$lib/api/types";
  import Spinner from "$lib/components/Spinner.svelte";

  const canManage = $derived(auth.isAdmin || auth.isModerator);
  const id = page.params.id!;
  const queryClient = useQueryClient();

  const gameQuery = createQuery(() => ({
    queryKey: ["games", id],
    queryFn: () => getGame(id),
  }));

  const teamsQuery = createQuery(() => ({
    queryKey: ["teams"],
    queryFn: listTeams,
    staleTime: 10 * 60_000,
  }));
  const teams: Team[] = $derived(teamsQuery.data ?? []);

  const updateMutation = createMutation(() => ({
    mutationFn: (data: UpdateGameRequest) => updateGame(id, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["games"] });
      goto(`/games/${id}`);
    },
  }));

  let game_state = $state<Game | null>(null);
  let sameTeamError = $state(false);

  function isoToLocalInput(iso: string): string {
    const d = new Date(iso);
    const pad = (n: number) => String(n).padStart(2, "0");
    return (
      `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}` +
      `T${pad(d.getHours())}:${pad(d.getMinutes())}`
    );
  }

  $effect(() => {
    const data = gameQuery.data as Game | null | undefined;
    if (data && !game_state) {
      game_state = { ...data, dateTime: isoToLocalInput(data.dateTime) };
    }
  });

  const isFinishedOrLive = $derived(
    game_state?.status === "finished" || game_state?.status === "live",
  );

  function handleSubmit(e: Event) {
    e.preventDefault();
    if (!game_state) return;
    if (game_state.homeTeamId === game_state.awayTeamId) {
      sameTeamError = true;
      return;
    }
    sameTeamError = false;
    const dateTimeIso = new Date(game_state.dateTime).toISOString();
    updateMutation.mutate({
      homeTeamId: game_state.homeTeamId,
      awayTeamId: game_state.awayTeamId,
      location: game_state.location,
      dateTime: dateTimeIso,
      cancelled: game_state.cancelled,
      homeScore: game_state.homeScore,
      awayScore: game_state.awayScore,
    });
  }
</script>

{#if !canManage}
  <p class="text-error-500 font-medium">
    Geen toegang. Admin- of moderatorrol vereist.
  </p>
{:else if gameQuery.isPending}
  <Spinner />
{:else if gameQuery.isError}
  <p class="text-error-500 text-sm">Kon wedstrijd niet laden</p>
{:else if !gameQuery.data}
  <p class="text-surface-400">Wedstrijd niet gevonden.</p>
{:else if game_state != null}
  <div class="max-w-lg">
    <a
      href="/games/{id}"
      class="text-sm text-primary-500 hover:underline mb-4 inline-block"
      >&larr; Terug naar wedstrijd</a
    >
    <h1 class="text-2xl font-bold mb-6">Wedstrijd bewerken</h1>
    <form onsubmit={handleSubmit} class="card p-6 space-y-4">
      <div>
        <label for="homeTeam" class="label-text">Thuisploeg</label>
        <select
          id="homeTeam"
          bind:value={game_state.homeTeamId}
          required
          class="select"
        >
          {#each teams as team}
            <option value={team.id}>{team.name}</option>
          {/each}
        </select>
      </div>
      <div>
        <label for="awayTeam" class="label-text">Uitploeg</label>
        <select
          id="awayTeam"
          bind:value={game_state.awayTeamId}
          required
          class="select"
        >
          {#each teams as team}
            <option value={team.id}>{team.name}</option>
          {/each}
        </select>
      </div>
      {#if sameTeamError}
        <p class="text-error-500 text-sm">
          Thuis- en uitploeg moeten verschillen.
        </p>
      {/if}
      <div>
        <label for="location" class="label-text">Locatie</label>
        <input
          id="location"
          type="text"
          bind:value={game_state.location}
          class="input"
        />
      </div>
      <div>
        <label for="dateTime" class="label-text">Datum & tijd</label>
        <input
          id="dateTime"
          type="datetime-local"
          bind:value={game_state.dateTime}
          required
          class="input"
        />
      </div>
      <label class="flex items-center gap-2">
        <input
          type="checkbox"
          bind:checked={game_state.cancelled}
          class="checkbox"
        />
        <span class="label-text">Afgelast</span>
      </label>
      {#if isFinishedOrLive && !game_state.cancelled}
        <div class="grid grid-cols-2 gap-4">
          <div>
            <label for="homeScore" class="label-text">Thuisscore</label>
            <input
              id="homeScore"
              type="number"
              bind:value={game_state.homeScore}
              min="0"
              required
              class="input"
            />
          </div>
          <div>
            <label for="awayScore" class="label-text">Uitscore</label>
            <input
              id="awayScore"
              type="number"
              bind:value={game_state.awayScore}
              min="0"
              required
              class="input"
            />
          </div>
        </div>
      {/if}
      <button type="submit" class="btn w-full preset-filled-primary-500">
        Wijzigingen opslaan
      </button>
    </form>
  </div>
{/if}

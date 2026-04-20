<script lang="ts">
  import { goto } from "$app/navigation";
  import { auth } from "$lib/state/auth.svelte";
  import { createGame } from "$lib/api/games";
  import { listTeams, createTeam } from "$lib/api/teams";
  import {
    createMutation,
    createQuery,
    useQueryClient,
  } from "@tanstack/svelte-query";
  import type { CreateGameRequest, Team } from "$lib/api/types";
  import Spinner from "$lib/components/Spinner.svelte";

  const canManage = $derived(auth.isAdmin || auth.isModerator);
  const queryClient = useQueryClient();

  let homeTeamId = $state("");
  let awayTeamId = $state("");
  let location = $state("");
  let dateTime = $state("");
  let newTeamName = $state("");
  let sameTeamError = $state(false);

  const teamsQuery = createQuery(() => ({
    queryKey: ["teams"],
    queryFn: listTeams,
    staleTime: 10 * 60_000,
  }));
  const teams: Team[] = $derived(teamsQuery.data ?? []);

  const createMut = createMutation(() => ({
    mutationFn: (data: CreateGameRequest) => createGame(data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["games"] });
      goto("/games");
    },
  }));

  const createTeamMut = createMutation(() => ({
    mutationFn: (name: string) => createTeam(name),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["teams"] });
      newTeamName = "";
    },
  }));

  function handleSubmit(e: Event) {
    e.preventDefault();
    if (homeTeamId === awayTeamId) {
      sameTeamError = true;
      return;
    }
    sameTeamError = false;
    createMut.mutate({
      homeTeamId,
      awayTeamId,
      location,
      dateTime: new Date(dateTime),
    });
  }

  function handleCreateTeam() {
    if (newTeamName.trim()) {
      createTeamMut.mutate(newTeamName.trim());
    }
  }
</script>

{#if canManage}
  <div class="max-w-lg">
    <a
      href="/games"
      class="text-sm text-primary-500 hover:underline mb-4 inline-block"
      >&larr; Alle wedstrijden</a
    >
    <h1 class="text-2xl font-bold mb-6">Nieuwe wedstrijd</h1>
    {#if teamsQuery.isPending}
      <Spinner />
    {:else if teamsQuery.isError}
      <p class="text-error-500 text-sm">Kon ploegen niet laden</p>
    {:else}
      <form onsubmit={handleSubmit} class="card p-6 space-y-4">
        <div>
          <label for="homeTeam" class="label-text">Thuisploeg</label>
          <select id="homeTeam" bind:value={homeTeamId} required class="select">
            <option value="" disabled>Kies thuisploeg</option>
            {#each teams as team}
              <option value={team.id}>{team.name}</option>
            {/each}
          </select>
        </div>
        <div>
          <label for="awayTeam" class="label-text">Uitploeg</label>
          <select id="awayTeam" bind:value={awayTeamId} required class="select">
            <option value="" disabled>Kies uitploeg</option>
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
            bind:value={location}
            required
            class="input"
          />
        </div>
        <div>
          <label for="dateTime" class="label-text">Datum & tijd</label>
          <input
            id="dateTime"
            type="datetime-local"
            bind:value={dateTime}
            required
            class="input"
          />
        </div>
        <button type="submit" class="btn w-full preset-filled-primary-500">
          Wedstrijd aanmaken
        </button>
      </form>

      <div class="card p-4 mt-6 space-y-2">
        <p class="text-sm font-medium">Nieuwe ploeg toevoegen</p>
        <div class="flex gap-2">
          <input
            type="text"
            bind:value={newTeamName}
            placeholder="Ploegnaam"
            class="input flex-1"
          />
          <button
            type="button"
            onclick={handleCreateTeam}
            class="btn preset-outlined-primary-500"
            disabled={!newTeamName.trim()}
          >
            + Toevoegen
          </button>
        </div>
      </div>
    {/if}
  </div>
{:else}
  <p class="text-error-500 font-medium">
    Geen toegang. Admin- of moderatorrol vereist.
  </p>
{/if}

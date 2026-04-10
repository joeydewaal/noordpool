<script lang="ts">
  import { page } from "$app/state";
  import { goto } from "$app/navigation";
  import { auth } from "$lib/state/auth.svelte";
  import { getGame, updateGame } from "$lib/api/games";
  import {
    createQuery,
    createMutation,
    useQueryClient,
  } from "@tanstack/svelte-query";
  import type { Game, UpdateGameRequest } from "$lib/api/types";

  const canManage = $derived(auth.isAdmin || auth.isModerator);
  const id = page.params.id!;
  const queryClient = useQueryClient();

  const gameQuery = createQuery(() => ({
    queryKey: ["games", id],
    queryFn: () => getGame(id),
  }));

  const updateMutation = createMutation(() => ({
    mutationFn: (data: UpdateGameRequest) => updateGame(id, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["games"] });
      goto(`/games/${id}`);
    },
  }));

  let game_state = $state<Game | null>(null);

  // `<input type="datetime-local">` only accepts the format
  // `YYYY-MM-DDTHH:mm` in the user's local time — it rejects ISO
  // strings with a `Z` / timezone suffix (which is what the API
  // returns). We convert on the way in here, and back to a UTC
  // ISO string in `handleSubmit`.
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

  // Server-derived; once the match window has closed `status` becomes
  // `finished` (or `cancelled`). The frontend never recomputes liveness.
  const isFinishedOrLive = $derived(
    game_state?.status === "finished" || game_state?.status === "live",
  );

  function handleSubmit(e: Event) {
    e.preventDefault();
    if (!game_state) return;
    // Convert the local-time `datetime-local` value back to a
    // UTC ISO string for the backend's `jiff::Timestamp`.
    const dateTimeIso = new Date(game_state.dateTime).toISOString();
    updateMutation.mutate({
      opponent: game_state.opponent,
      location: game_state.location,
      dateTime: dateTimeIso,
      homeAway: game_state.homeAway,
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
  <p class="text-surface-400">Laden...</p>
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
        <label for="opponent" class="label-text">Tegenstander</label>
        <input
          id="opponent"
          type="text"
          bind:value={game_state.opponent}
          required
          class="input"
        />
      </div>
      <div>
        <label for="location" class="label-text">Locatie</label>
        <input
          id="location"
          type="text"
          bind:value={game_state.location}
          required
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
      <fieldset>
        <legend class="label-text mb-2">Thuis / Uit</legend>
        <div class="flex gap-4">
          <label class="flex items-center gap-2">
            <input
              type="radio"
              bind:group={game_state.homeAway}
              value="home"
              class="radio"
            />
            <span class="text-sm">Thuis</span>
          </label>
          <label class="flex items-center gap-2">
            <input
              type="radio"
              bind:group={game_state.homeAway}
              value="away"
              class="radio"
            />
            <span class="text-sm">Uit</span>
          </label>
        </div>
      </fieldset>
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

<script lang="ts">
  import { page } from "$app/state";
  import { auth } from "$lib/state/auth.svelte";
  import { getGame, adjustScore } from "$lib/api/games";
  import { createGameEvent, deleteGameEvent } from "$lib/api/events";
  import { getPlayers } from "$lib/api/players";
  import {
    createQuery,
    createMutation,
    useQueryClient,
  } from "@tanstack/svelte-query";
  import type {
    EventType,
    CreateGameEventRequest,
    Player,
    LivePoll,
    GameStatus,
    Game,
    ScoreSide,
    GameEvent,
  } from "$lib/api/types";
  import { startLiveMatchStream } from "$lib/live-match.svelte";
  import Spinner from "$lib/components/Spinner.svelte";

  const id = page.params.id!;
  const queryClient = useQueryClient();

  const canManage = $derived(auth.isAdmin || auth.isModerator);

  const gameQuery = createQuery(() => ({
    queryKey: ["games", id],
    queryFn: () => getGame(id),
  }));

  let liveOverlay = $state<LivePoll | null>(null);

  const game = $derived.by((): Game | null | undefined => {
    return gameQuery.data as unknown as Game | null | undefined;
  });

  const status: GameStatus = $derived(
    liveOverlay?.status ?? game?.status ?? "scheduled",
  );
  const homeScore = $derived(liveOverlay?.homeScore ?? game?.homeScore ?? 0);
  const awayScore = $derived(liveOverlay?.awayScore ?? game?.awayScore ?? 0);
  const events = $derived(liveOverlay?.events ?? game?.events ?? []);

  // Derive which side the current user is on (null if not a player in this game)
  const ownSide: ScoreSide | null = $derived.by(() => {
    const teamId = auth.teamId;
    if (!teamId || !game) return null;
    if (game.homeTeam.id === teamId) return "home";
    if (game.awayTeam.id === teamId) return "away";
    return null;
  });

  $effect(() => {
    if (status !== "live") return;
    return startLiveMatchStream(id, {
      onSnapshot: (snapshot) => {
        liveOverlay = snapshot;
      },
      onScoreUpdate: ({ home, away, version, updatedAt }) => {
        if (!liveOverlay) return;
        liveOverlay = {
          ...liveOverlay,
          homeScore: home,
          awayScore: away,
          version,
          updatedAt,
        };
      },
      onEventAdded: (event: GameEvent) => {
        if (!liveOverlay) return;
        liveOverlay = {
          ...liveOverlay,
          events: [...liveOverlay.events, event],
        };
      },
      onEventDeleted: (eventId) => {
        if (!liveOverlay) return;
        liveOverlay = {
          ...liveOverlay,
          events: liveOverlay.events.filter((e) => e.id !== eventId),
        };
      },
      onStatusChange: (newStatus) => {
        if (!liveOverlay) return;
        liveOverlay = { ...liveOverlay, status: newStatus };
      },
    });
  });

  const scoreMutation = createMutation(() => ({
    mutationFn: ({ side, delta }: { side: ScoreSide; delta: 1 | -1 }) =>
      adjustScore(id, side, delta),
    onSuccess: (data) => {
      liveOverlay = data;
    },
  }));

  const playersQuery = createQuery(() => ({
    queryKey: ["players"],
    queryFn: getPlayers,
    enabled: canManage,
  }));

  const addEventMutation = createMutation(() => ({
    mutationFn: (data: CreateGameEventRequest) => createGameEvent(id, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["games", id] });
    },
  }));

  const deleteEventMutation = createMutation(() => ({
    mutationFn: (eventId: string) => deleteGameEvent(id, eventId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["games", id] });
    },
  }));

  const eventLabels: Record<EventType, string> = {
    goal: "Doelpunt",
    own_goal: "Eigen doelpunt",
    assist: "Assist",
    yellow_card: "Gele kaart",
    red_card: "Rode kaart",
  };

  const eventIcons: Record<EventType, string> = {
    goal: "\u26BD",
    own_goal: "\uD83D\uDD34\u26BD",
    assist: "\uD83D\uDC5F",
    yellow_card: "\uD83D\uDFE8",
    red_card: "\uD83D\uDFE5",
  };

  let newPlayerId = $state("");
  let newEventType: EventType = $state("goal");
  let newMinute = $state(1);

  function playerName(player: Player): string {
    return `${player.firstName} ${player.lastName}`.trim();
  }

  function handleAddEvent() {
    if (!newPlayerId) return;
    addEventMutation.mutate(
      {
        playerId: newPlayerId,
        eventType: newEventType,
        minute: newMinute,
      },
      {
        onSuccess: () => {
          newPlayerId = "";
          newEventType = "goal";
          newMinute = 1;
        },
      },
    );
  }

  function handleDeleteEvent(eventId: string) {
    deleteEventMutation.mutate(eventId);
  }

  function formatDate(dateTime: string): string {
    return new Date(dateTime).toLocaleDateString("nl-NL", {
      weekday: "long",
      day: "numeric",
      month: "long",
      year: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  }
</script>

{#if gameQuery.isPending}
  <Spinner />
{:else if gameQuery.data}
  {@const g = gameQuery.data}
  <div class="max-w-lg">
    <button
      onclick={() => history.back()}
      class="text-sm text-primary-500 hover:underline mb-4 inline-block"
      >&larr; Alle wedstrijden</button
    >
    <div class="card p-6">
      <div class="flex items-center justify-between mb-2">
        <h1 class="text-2xl font-bold">
          {g.homeTeam.name} vs {g.awayTeam.name}
        </h1>
        {#if ownSide}
          <span
            class="chip {ownSide === 'home'
              ? 'preset-filled-success-500'
              : 'preset-filled-secondary-500'}"
          >
            {ownSide === "home" ? "thuis" : "uit"}
          </span>
        {/if}
      </div>

      <div class="text-sm text-surface-400 space-y-1 mb-4">
        <div>{formatDate(g.dateTime)}</div>
        <div>{g.location}</div>
        <div>
          Status:
          <span
            class="font-medium {status === 'finished'
              ? 'text-success-500'
              : status === 'cancelled'
                ? 'text-error-500'
                : status === 'live'
                  ? 'text-warning-500'
                  : 'text-primary-500'}"
          >
            {status === "scheduled"
              ? "gepland"
              : status === "finished"
                ? "gespeeld"
                : status === "live"
                  ? "bezig"
                  : "afgelast"}
          </span>
          {#if status === "live"}
            <span class="chip preset-filled-error-500 animate-pulse ml-2">
              <span class="inline-block w-2 h-2 rounded-full bg-white mr-1"
              ></span>
              LIVE
            </span>
          {/if}
        </div>
      </div>

      {#if status === "finished" || status === "live"}
        <div class="card preset-tonal-surface p-4 text-center">
          <div class="text-lg">
            <span class="font-bold">{g.homeTeam.name} {homeScore}</span>
            <span class="text-surface-500 mx-2">-</span>
            <span class="font-bold">{awayScore} {g.awayTeam.name}</span>
          </div>
        </div>
      {/if}

      {#if status === "live" && canManage}
        <div
          class="mt-4 card preset-tonal-warning p-4"
          aria-label="Score adjuster"
        >
          <h3 class="text-sm font-semibold mb-3">Score aanpassen</h3>
          {#if ownSide}
            <p class="text-xs text-surface-400 mb-3">
              Eigen doelpunten registreer je hieronder als gebeurtenis.
            </p>
          {/if}
          <div class="grid grid-cols-2 gap-4">
            {#each [{ side: "home" as ScoreSide, name: g.homeTeam.name, score: homeScore }, { side: "away" as ScoreSide, name: g.awayTeam.name, score: awayScore }] as { side, name, score }}
              {@const isOwn = ownSide === side}
              <div class="flex flex-col items-center gap-1">
                <div class="text-xs text-surface-400">
                  {name}{isOwn ? " (eigen)" : ""}
                </div>
                <div class="text-2xl font-bold">{score}</div>
                <div class="flex gap-2">
                  <button
                    type="button"
                    class="btn btn-sm preset-filled-error-500"
                    onclick={() => scoreMutation.mutate({ side, delta: -1 })}
                    disabled={score === 0 || scoreMutation.isPending}
                    aria-label={isOwn
                      ? "Eigen doelpunt intrekken"
                      : "Doelpunt tegenstander intrekken"}
                  >
                    &minus;
                  </button>
                  <button
                    type="button"
                    class="btn btn-sm preset-filled-success-500"
                    onclick={() => scoreMutation.mutate({ side, delta: 1 })}
                    disabled={scoreMutation.isPending}
                    aria-label={isOwn
                      ? "Eigen doelpunt"
                      : "Doelpunt tegenstander"}
                  >
                    +
                  </button>
                </div>
              </div>
            {/each}
          </div>
        </div>
      {/if}

      {#if status === "finished" || status === "live"}
        <div
          class="mt-6 pt-4 border-t border-surface-200 dark:border-surface-800"
        >
          <h2 class="text-lg font-bold mb-3">Wedstrijdverloop</h2>

          {#if events.length === 0}
            <p class="text-sm text-surface-400">
              Geen gebeurtenissen geregistreerd.
            </p>
          {:else}
            <div class="space-y-2">
              {#each events as event}
                <div class="flex items-center gap-3 text-sm">
                  <span
                    class="inline-flex items-center justify-center w-10 h-6 preset-tonal-surface font-mono text-xs rounded"
                  >
                    {event.minute}'
                  </span>
                  <span class="text-base">{eventIcons[event.eventType]}</span>
                  <span class="font-medium">{playerName(event.player)}</span>
                  <span class="text-surface-400"
                    >{eventLabels[event.eventType]}</span
                  >
                  {#if canManage}
                    <button
                      onclick={() => handleDeleteEvent(event.id)}
                      class="ml-auto text-error-400 hover:text-error-500 text-xs"
                      title="Verwijderen">&times;</button
                    >
                  {/if}
                </div>
              {/each}
            </div>
          {/if}

          {#if canManage}
            <div
              class="mt-4 pt-3 border-t border-surface-200 dark:border-surface-800"
            >
              <h3 class="text-sm font-semibold mb-2">Gebeurtenis toevoegen</h3>
              <form
                onsubmit={(e) => {
                  e.preventDefault();
                  handleAddEvent();
                }}
                class="flex flex-wrap gap-2 items-end"
              >
                <select
                  bind:value={newPlayerId}
                  class="select flex-1 min-w-[140px]"
                >
                  <option value="">Speler...</option>
                  {#each (playersQuery.data ?? []).filter((p) => p.active) as p}
                    <option value={p.id}>{playerName(p)}</option>
                  {/each}
                </select>
                <select bind:value={newEventType} class="select">
                  <option value="goal">Doelpunt</option>
                  <option value="own_goal">Eigen doelpunt</option>
                  <option value="assist">Assist</option>
                  <option value="yellow_card">Gele kaart</option>
                  <option value="red_card">Rode kaart</option>
                </select>
                <input
                  type="number"
                  bind:value={newMinute}
                  min="1"
                  max="120"
                  class="input w-16"
                  placeholder="min"
                />
                <button
                  type="submit"
                  disabled={!newPlayerId}
                  class="btn btn-sm preset-filled-primary-500 disabled:opacity-50"
                >
                  Toevoegen
                </button>
              </form>
            </div>
          {/if}
        </div>
      {/if}

      {#if canManage}
        <div
          class="mt-6 pt-4 border-t border-surface-200 dark:border-surface-800"
        >
          <a
            href="/games/{g.id}/edit"
            class="btn btn-sm preset-filled-primary-500"
          >
            Bewerken
          </a>
        </div>
      {/if}
    </div>
  </div>
{:else if gameQuery.isError}
  <p class="text-surface-400">Wedstrijd niet gevonden.</p>
{/if}

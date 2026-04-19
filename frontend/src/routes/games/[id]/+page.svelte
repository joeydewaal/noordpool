<script lang="ts">
  import { page } from "$app/state";
  import { auth } from "$lib/state/auth.svelte";
  import { getGame } from "$lib/api/games";
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
    GameEvent,
  } from "$lib/api/types";
  import { startLiveMatchStream } from "$lib/live-match.svelte";
  import { Dialog } from "@skeletonlabs/skeleton-svelte";
  import { toaster } from "$lib/state/toaster";
  import Spinner from "$lib/components/Spinner.svelte";
  import PlayerAvatar from "$lib/components/PlayerAvatar.svelte";

  const id = page.params.id!;
  const queryClient = useQueryClient();

  const canManage = $derived(auth.isAdmin || auth.isModerator);

  const gameQuery = createQuery(() => ({
    queryKey: ["games", id],
    queryFn: () => getGame(id),
  }));

  let liveOverlay = $state<LivePoll | null>(null);
  let showCommands = $state(false);

  const game = $derived.by((): Game | null | undefined => {
    return gameQuery.data as unknown as Game | null | undefined;
  });

  const status: GameStatus = $derived(
    liveOverlay?.status ?? game?.status ?? "scheduled",
  );
  const homeScore = $derived(liveOverlay?.homeScore ?? game?.homeScore ?? 0);
  const awayScore = $derived(liveOverlay?.awayScore ?? game?.awayScore ?? 0);
  const events = $derived(liveOverlay?.events ?? game?.events ?? []);

  // Auto-computed match minute (1–90), updated every 30 s while live.
  let now = $state(Date.now());
  $effect(() => {
    if (status !== "live") return;
    const timer = setInterval(() => (now = Date.now()), 30_000);
    return () => clearInterval(timer);
  });
  const matchMinute = $derived(
    Math.min(
      90,
      Math.max(
        1,
        Math.floor((now - new Date(game?.dateTime ?? 0).getTime()) / 60_000),
      ),
    ),
  );

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

  // Command panel state — step 1: pick player, step 2: pick action.
  let selectedPlayer = $state<Player | null>(null);
  let selectedAction: EventType = $state("goal");
  let newMinute = $state(1);

  $effect(() => {
    if (showCommands) newMinute = matchMinute;
  });

  const allPlayers = $derived(
    (playersQuery.data ?? []).filter((p) => p.active),
  );
  const homePlayers = $derived(
    allPlayers.filter((p) => p.teamId === game?.homeTeamId),
  );
  const awayPlayers = $derived(
    allPlayers.filter((p) => p.teamId === game?.awayTeamId),
  );

  function selectPlayer(p: Player) {
    selectedPlayer = p;
    selectedAction = "goal";
    newMinute = matchMinute;
  }

  function handleAddEvent() {
    if (!selectedPlayer) return;
    addEventMutation.mutate(
      {
        playerId: selectedPlayer.id,
        eventType: selectedAction,
        minute: newMinute,
      },
      {
        onSuccess: () => {
          const label = eventLabels[selectedAction];
          showCommands = false;
          selectedPlayer = null;
          selectedAction = "goal";
          newMinute = matchMinute;
          toaster.success({ title: `${label} toegevoegd` });
        },
      },
    );
  }

  function handleDeleteEvent(eventId: string) {
    deleteEventMutation.mutate(eventId);
  }

  function playerName(player: Player): string {
    return `${player.firstName} ${player.lastName}`.trim();
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

  // Group events: pair each assist at the same minute as the preceding goal.
  type EventGroup = { main: GameEvent; assist?: GameEvent };
  const groupedEvents = $derived.by((): EventGroup[] => {
    const sorted = [...events].sort((a, b) => a.minute - b.minute);
    const groups: EventGroup[] = [];
    for (const ev of sorted) {
      if (ev.eventType === "assist") {
        const last = groups[groups.length - 1];
        if (last && last.main.minute === ev.minute && !last.assist) {
          last.assist = ev;
          continue;
        }
      }
      groups.push({ main: ev });
    }
    return groups;
  });

  const ownSide = $derived.by(() => {
    const teamId = auth.teamId;
    if (!teamId || !game) return null;
    if (game.homeTeam.id === teamId) return "home";
    if (game.awayTeam.id === teamId) return "away";
    return null;
  });
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
      <!-- Scoreboard header -->
      <div class="text-center mb-1">
        {#if ownSide}
          <span
            class="chip {ownSide === 'home'
              ? 'preset-filled-success-500'
              : 'preset-filled-secondary-500'} mb-2"
          >
            {ownSide === "home" ? "thuis" : "uit"}
          </span>
        {/if}
      </div>

      <div class="grid grid-cols-[1fr_auto_1fr] items-center gap-2 mb-3">
        <div class="text-center">
          <p class="text-sm font-semibold leading-tight">{g.homeTeam.name}</p>
        </div>
        {#if status === "finished" || status === "live"}
          <div class="flex items-center gap-2 px-2">
            <span class="text-4xl font-black tabular-nums">{homeScore}</span>
            <span class="text-2xl text-surface-400">-</span>
            <span class="text-4xl font-black tabular-nums">{awayScore}</span>
          </div>
        {:else}
          <div class="text-center px-2">
            <p class="text-sm text-surface-400">vs</p>
          </div>
        {/if}
        <div class="text-center">
          <p class="text-sm font-semibold leading-tight">{g.awayTeam.name}</p>
        </div>
      </div>

      <!-- Status row -->
      <div class="text-center text-sm text-surface-400 space-y-1 mb-4">
        <div>{formatDate(g.dateTime)}</div>
        <div>{g.location}</div>
        {#if status === "live"}
          <div class="flex items-center justify-center gap-2">
            <span class="chip preset-filled-error-500 animate-pulse">
              <span class="inline-block w-2 h-2 rounded-full bg-white mr-1"
              ></span>
              LIVE
            </span>
          </div>
        {:else}
          <div>
            Status:
            <span
              class="font-medium {status === 'finished'
                ? 'text-success-500'
                : status === 'cancelled'
                  ? 'text-error-500'
                  : 'text-primary-500'}"
            >
              {status === "scheduled"
                ? "gepland"
                : status === "finished"
                  ? "gespeeld"
                  : "afgelast"}
            </span>
          </div>
        {/if}
      </div>

      <!-- Event timeline -->
      {#if status === "finished" || status === "live"}
        <div
          class="mt-4 pt-4 border-t border-surface-200 dark:border-surface-800"
        >
          <h2 class="text-base font-bold mb-3">Wedstrijdverloop</h2>

          {#if groupedEvents.length === 0}
            <p class="text-sm text-surface-400">
              Geen gebeurtenissen geregistreerd.
            </p>
          {:else}
            <div class="space-y-1">
              {#each groupedEvents as group}
                <div class="flex items-center gap-3 text-sm">
                  <span
                    class="inline-flex items-center justify-center w-10 h-6 preset-tonal-surface font-mono text-xs rounded shrink-0"
                  >
                    {group.main.minute}'
                  </span>
                  <span class="text-base"
                    >{eventIcons[group.main.eventType]}</span
                  >
                  <span class="font-medium"
                    >{playerName(group.main.player)}</span
                  >
                  <span class="text-surface-400"
                    >{eventLabels[group.main.eventType]}</span
                  >
                  {#if canManage}
                    <button
                      onclick={() => handleDeleteEvent(group.main.id)}
                      class="ml-auto text-error-400 hover:text-error-500 text-xs shrink-0"
                      title="Verwijderen">&times;</button
                    >
                  {/if}
                </div>
                {#if group.assist}
                  <div class="flex items-center gap-3 text-sm pl-4">
                    <span
                      class="inline-flex items-center justify-center w-10 h-5 font-mono text-xs rounded shrink-0 text-surface-400"
                    >
                      {group.assist.minute}'
                    </span>
                    <span class="text-base"
                      >{eventIcons[group.assist.eventType]}</span
                    >
                    <span class="font-medium text-surface-400"
                      >{playerName(group.assist.player)}</span
                    >
                    <span class="text-surface-400"
                      >{eventLabels[group.assist.eventType]}</span
                    >
                    {#if canManage}
                      <button
                        onclick={() => handleDeleteEvent(group.assist!.id)}
                        class="ml-auto text-error-400 hover:text-error-500 text-xs shrink-0"
                        title="Verwijderen">&times;</button
                      >
                    {/if}
                  </div>
                {/if}
              {/each}
            </div>
          {/if}
        </div>
      {/if}

      <!-- Moderator controls -->
      {#if canManage}
        <div
          class="mt-5 pt-4 border-t border-surface-200 dark:border-surface-800 flex items-center justify-between"
        >
          <a
            href="/games/{g.id}/edit"
            class="btn btn-sm preset-outlined-surface-500">Bewerken</a
          >
          {#if status === "live"}
            <Dialog
              open={showCommands}
              onOpenChange={(e) => {
                showCommands = e.open;
                if (!e.open) selectedPlayer = null;
              }}
            >
              <Dialog.Trigger
                type="button"
                class="btn btn-sm preset-outlined-warning-500"
              >
                Wedstrijdbeheer
              </Dialog.Trigger>

              <Dialog.Backdrop class="fixed inset-0 bg-black/50 z-40" />
              <Dialog.Positioner
                class="fixed inset-0 flex items-center justify-center z-50 p-4"
              >
                <Dialog.Content
                  class="card bg-surface-100-900 p-5 w-full max-w-md shadow-xl flex flex-col h-[480px]"
                  aria-label="Wedstrijdbeheer"
                >
                  <div class="flex items-center justify-between mb-4 shrink-0">
                    <Dialog.Title class="text-base font-bold"
                      >Wedstrijdbeheer</Dialog.Title
                    >
                    <Dialog.CloseTrigger
                      type="button"
                      class="btn btn-sm preset-outlined-surface-500"
                      >&times;</Dialog.CloseTrigger
                    >
                  </div>

                  {#if !selectedPlayer}
                    <!-- Step 1: pick a player -->
                    <div class="grid grid-cols-2 gap-4 flex-1 min-h-0">
                      {#each [{ label: g.homeTeam.name, players: homePlayers }, { label: g.awayTeam.name, players: awayPlayers }] as team}
                        <div class="flex flex-col min-h-0">
                          <p
                            class="text-xs font-semibold text-surface-400 mb-2 shrink-0"
                          >
                            {team.label}
                          </p>
                          {#if team.players.length === 0}
                            <p class="text-xs text-surface-400 italic">
                              Geen spelers
                            </p>
                          {:else}
                            <div
                              class="flex flex-col gap-1 overflow-y-auto pr-1"
                            >
                              {#each team.players as p}
                                <button
                                  type="button"
                                  class="btn btn-sm preset-outlined-surface-500 flex items-center gap-2 justify-start shrink-0"
                                  onclick={() => selectPlayer(p)}
                                >
                                  <PlayerAvatar
                                    avatarUrl={p.user?.avatarUrl}
                                    shirtNumber={p.shirtNumber}
                                    size="sm"
                                  />
                                  <span class="text-xs truncate"
                                    >{playerName(p)}</span
                                  >
                                </button>
                              {/each}
                            </div>
                          {/if}
                        </div>
                      {/each}
                    </div>
                  {:else}
                    <!-- Step 2: pick action -->
                    <div class="flex flex-col flex-1 min-h-0">
                      <div class="mb-3 flex items-center gap-2 shrink-0">
                        <button
                          type="button"
                          class="btn btn-sm preset-outlined-surface-500"
                          onclick={() => (selectedPlayer = null)}
                          >&larr; Terug</button
                        >
                        <span class="text-sm font-semibold"
                          >{playerName(selectedPlayer)}</span
                        >
                      </div>

                      <div class="flex flex-wrap gap-2 flex-1 content-start">
                        {#each ["goal", "own_goal", "assist", "yellow_card", "red_card"] as EventType[] as action}
                          <button
                            type="button"
                            class="btn btn-sm {selectedAction === action
                              ? 'preset-filled-warning-500'
                              : 'preset-outlined-surface-500'}"
                            onclick={() => (selectedAction = action)}
                          >
                            {eventIcons[action]}
                            {eventLabels[action]}
                          </button>
                        {/each}
                      </div>

                      <div
                        class="flex items-center justify-between gap-3 shrink-0 pt-4 border-t border-surface-200 dark:border-surface-800"
                      >
                        <div class="flex items-center gap-2">
                          <label
                            class="text-xs text-surface-400"
                            for="event-minute">Minuut</label
                          >
                          <input
                            id="event-minute"
                            type="number"
                            bind:value={newMinute}
                            min="1"
                            max="120"
                            class="input w-16 text-sm"
                          />
                        </div>
                        <button
                          type="button"
                          class="btn btn-sm preset-filled-primary-500"
                          disabled={addEventMutation.isPending}
                          onclick={handleAddEvent}
                        >
                          {#if addEventMutation.isPending}
                            <Spinner size="sm" />
                          {:else}
                            Toevoegen
                          {/if}
                        </button>
                      </div>
                    </div>
                  {/if}
                </Dialog.Content>
              </Dialog.Positioner>
            </Dialog>
          {/if}
        </div>
      {/if}
    </div>
  </div>
{:else if gameQuery.isError}
  <p class="text-surface-400">Wedstrijd niet gevonden.</p>
{/if}

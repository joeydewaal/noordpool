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
    } from "$lib/api/types";
    import { getGameStatus } from "$lib/api/types";

    const id = page.params.id!;
    const queryClient = useQueryClient();

    const canManage = $derived(auth.isAdmin || auth.isModerator);

    const gameQuery = createQuery(() => ({
        queryKey: ["games", id],
        queryFn: () => getGame(id),
    }));

    const status = $derived(gameQuery.data ? getGameStatus(gameQuery.data) : 'scheduled');

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
        assist: "Assist",
        yellow_card: "Gele kaart",
        red_card: "Rode kaart",
    };

    const eventIcons: Record<EventType, string> = {
        goal: "\u26BD",
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

{#if gameQuery.data}
    <div class="max-w-lg">
        <button
            onclick={() => history.back()}
            class="text-sm text-primary-500 hover:underline mb-4 inline-block"
            >&larr; Alle wedstrijden</button
        >
        <div class="card p-6">
            <div class="flex items-center justify-between mb-2">
                <h1 class="text-2xl font-bold">
                    vs {gameQuery.data.opponent}
                </h1>
                <span
                    class="chip {gameQuery.data.homeAway === 'home'
                        ? 'preset-filled-success-500'
                        : 'preset-filled-secondary-500'}"
                >
                    {gameQuery.data.homeAway === "home" ? "thuis" : "uit"}
                </span>
            </div>

            <div class="text-sm text-surface-400 space-y-1 mb-4">
                <div>{formatDate(gameQuery.data.dateTime)}</div>
                <div>{gameQuery.data.location}</div>
                <div>
                    Status:
                    <span
                        class="font-medium {status === 'completed'
                            ? 'text-success-500'
                            : status === 'cancelled'
                              ? 'text-error-500'
                              : status === 'playing'
                                ? 'text-warning-500'
                                : 'text-primary-500'}"
                    >
                        {status === "scheduled"
                            ? "gepland"
                            : status === "completed"
                              ? "gespeeld"
                              : status === "playing"
                                ? "bezig"
                                : "afgelast"}
                    </span>
                </div>
            </div>

            {#if (status === "completed" || status === "playing") && gameQuery.data.homeScore !== null}
                <div class="card preset-tonal-surface p-4 text-center">
                    {#if gameQuery.data.homeAway === "home"}
                        <div class="text-lg">
                            <span class="font-bold"
                                >Noordpool {gameQuery.data.homeScore}</span
                            >
                            <span class="text-surface-500 mx-2">-</span>
                            <span class="font-bold"
                                >{gameQuery.data.awayScore}
                                {gameQuery.data.opponent}</span
                            >
                        </div>
                    {:else}
                        <div class="text-lg">
                            <span class="font-bold"
                                >{gameQuery.data.opponent}
                                {gameQuery.data.homeScore}</span
                            >
                            <span class="text-surface-500 mx-2">-</span>
                            <span class="font-bold"
                                >{gameQuery.data.awayScore} Noordpool</span
                            >
                        </div>
                    {/if}
                </div>
            {/if}

            {#if status === "completed" || status === "playing"}
                <div
                    class="mt-6 pt-4 border-t border-surface-200 dark:border-surface-800"
                >
                    <h2 class="text-lg font-bold mb-3">Wedstrijdverloop</h2>

                    {#if !gameQuery.data?.events || gameQuery.data.events.length === 0}
                        <p class="text-sm text-surface-400">
                            Geen gebeurtenissen geregistreerd.
                        </p>
                    {:else}
                        <div class="space-y-2">
                            {#each gameQuery.data.events as event}
                                <div class="flex items-center gap-3 text-sm">
                                    <span
                                        class="inline-flex items-center justify-center w-10 h-6 preset-tonal-surface font-mono text-xs rounded"
                                    >
                                        {event.minute}'
                                    </span>
                                    <span class="text-base"
                                        >{eventIcons[event.eventType]}</span
                                    >
                                    <span class="font-medium"
                                        >{playerName(event.player)}</span
                                    >
                                    <span class="text-surface-400"
                                        >{eventLabels[event.eventType]}</span
                                    >
                                    {#if canManage}
                                        <button
                                            onclick={() =>
                                                handleDeleteEvent(event.id)}
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
                            <h3 class="text-sm font-semibold mb-2">
                                Gebeurtenis toevoegen
                            </h3>
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
                                        <option value={p.id}>{p.name}</option>
                                    {/each}
                                </select>
                                <select
                                    bind:value={newEventType}
                                    class="select"
                                >
                                    <option value="goal">Doelpunt</option>
                                    <option value="assist">Assist</option>
                                    <option value="yellow_card"
                                        >Gele kaart</option
                                    >
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
                        href="/games/{gameQuery.data.id}/edit"
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

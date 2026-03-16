<script lang="ts">
    import { onMount } from "svelte";
    import { page } from "$app/state";
    import { auth } from "$lib/state/auth.svelte.js";
    import { getGame } from "$lib/api/games.js";
    import {
        getGameEvents,
        createGameEvent,
        deleteGameEvent,
    } from "$lib/api/events.js";
    import { getPlayers } from "$lib/api/players.js";
    import type { Game, GameEvent, Player, EventType } from "$lib/api/types.js";

    let game: Game | null = $state(null);
    let events: GameEvent[] = $state([]);
    let players: Player[] = $state([]);

    let newPlayerId = $state("");
    let newEventType: EventType = $state("goal");
    let newMinute = $state(1);

    const canManage = $derived(auth.isAdmin || auth.isModerator);

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

    function playerName(playerId: string): string {
        return players.find((p) => p.id === playerId)?.name ?? "Onbekend";
    }

    async function reloadEvents() {
        if (game) events = await getGameEvents(game.id);
    }

    async function handleAddEvent() {
        if (!game || !newPlayerId) return;
        await createGameEvent(game.id, {
            playerId: newPlayerId,
            eventType: newEventType,
            minute: newMinute,
        });
        await reloadEvents();
        newPlayerId = "";
        newEventType = "goal";
        newMinute = 1;
    }

    async function handleDeleteEvent(eventId: string) {
        if (!game) return;
        await deleteGameEvent(game.id, eventId);
        await reloadEvents();
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

    onMount(async () => {
        [game, players] = await Promise.all([
            getGame(page.params.id),
            getPlayers(),
        ]);
        if (game) events = await getGameEvents(game.id);
    });
</script>

{#if game}
    <div class="max-w-lg">
        <a
            href="/games"
            class="text-sm text-primary-500 hover:underline mb-4 inline-block"
            >&larr; Alle wedstrijden</a
        >
        <div class="card p-6">
            <div class="flex items-center justify-between mb-2">
                <h1 class="text-2xl font-bold">
                    vs {game.opponent}
                </h1>
                <span
                    class="chip {game.homeAway === 'home'
                        ? 'preset-filled-success-500'
                        : 'preset-filled-secondary-500'}"
                >
                    {game.homeAway === "home" ? "thuis" : "uit"}
                </span>
            </div>

            <div class="text-sm text-surface-400 space-y-1 mb-4">
                <div>{formatDate(game.dateTime)}</div>
                <div>{game.location}</div>
                <div>
                    Status:
                    <span
                        class="font-medium {game.status === 'completed'
                            ? 'text-success-500'
                            : game.status === 'cancelled'
                              ? 'text-error-500'
                              : 'text-primary-500'}"
                    >
                        {game.status === "scheduled"
                            ? "gepland"
                            : game.status === "completed"
                              ? "gespeeld"
                              : "afgelast"}
                    </span>
                </div>
            </div>

            {#if game.status === "completed" && game.homeScore !== null}
                <div class="card preset-tonal-surface p-4 text-center">
                    {#if game.homeAway === "home"}
                        <div class="text-lg">
                            <span class="font-bold"
                                >Noordpool {game.homeScore}</span
                            >
                            <span class="text-surface-500 mx-2">-</span>
                            <span class="font-bold"
                                >{game.awayScore} {game.opponent}</span
                            >
                        </div>
                    {:else}
                        <div class="text-lg">
                            <span class="font-bold"
                                >{game.opponent} {game.homeScore}</span
                            >
                            <span class="text-surface-500 mx-2">-</span>
                            <span class="font-bold"
                                >{game.awayScore} Noordpool</span
                            >
                        </div>
                    {/if}
                </div>
            {/if}

            {#if game.status === "completed"}
                <div class="mt-6 pt-4 border-t border-surface-200 dark:border-surface-800">
                    <h2 class="text-lg font-bold mb-3">
                        Wedstrijdverloop
                    </h2>

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
                                    <span class="text-base"
                                        >{eventIcons[event.eventType]}</span
                                    >
                                    <span class="font-medium"
                                        >{playerName(event.playerId)}</span
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
                        <div class="mt-4 pt-3 border-t border-surface-200 dark:border-surface-800">
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
                                    {#each players.filter((p) => p.active) as p}
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
                <div class="mt-6 pt-4 border-t border-surface-200 dark:border-surface-800">
                    <a
                        href="/games/{game.id}/edit"
                        class="btn btn-sm preset-filled-primary-500"
                    >
                        Bewerken
                    </a>
                </div>
            {/if}
        </div>
    </div>
{:else}
    <p class="text-surface-400">Wedstrijd niet gevonden.</p>
{/if}

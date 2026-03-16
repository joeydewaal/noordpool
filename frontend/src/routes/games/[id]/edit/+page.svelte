<script lang="ts">
    import { onMount } from "svelte";
    import { page } from "$app/state";
    import { goto } from "$app/navigation";
    import { auth } from "$lib/state/auth.svelte.ts";
    import { getGame, updateGame } from "$lib/api/games.ts";
    import type { Game } from "$lib/api/types.ts";

    const canManage = $derived(auth.isAdmin || auth.isModerator);

    let game_state: Game | null = $state(null);
    let loaded = $state(false);

    async function handleSubmit(e: Event) {
        e.preventDefault();
        await updateGame(page.params.id, {
            opponent: game_state?.opponent,
            location: game_state?.location,
            dateTime: game_state?.location,
            homeAway: game_state?.homeAway,
            status: game_state?.status,
            homeScore:
                game_state?.status === "completed"
                    ? game_state.homeScore
                    : null,
            awayScore:
                game_state?.status === "completed"
                    ? game_state.awayScore
                    : null,
        });
        goto(`/games/${page.params.id}`);
    }

    onMount(async () => {
        const game = await getGame(page.params.id);
        if (game) {
            loaded = true;
        }
    });
</script>

{#if !canManage}
    <p class="text-error-500 font-medium">
        Geen toegang. Admin- of moderatorrol vereist.
    </p>
{:else if !loaded}
    <p class="text-surface-400">Wedstrijd niet gevonden.</p>
{:else if game_state != null}
    <div class="max-w-lg">
        <a
            href="/games/{page.params.id}"
            class="text-sm text-primary-500 hover:underline mb-4 inline-block"
            >&larr; Terug naar wedstrijd</a
        >
        <h1 class="text-2xl font-bold mb-6">
            Wedstrijd bewerken
        </h1>
        <form
            onsubmit={handleSubmit}
            class="card p-6 space-y-4"
        >
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
            <div>
                <label for="status" class="label-text">Status</label>
                <select
                    id="status"
                    bind:value={game_state.status}
                    class="select"
                >
                    <option value="scheduled">Gepland</option>
                    <option value="completed">Gespeeld</option>
                    <option value="cancelled">Afgelast</option>
                </select>
            </div>
            {#if game_state.status === "completed"}
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
            <button
                type="submit"
                class="btn w-full preset-filled-primary-500"
            >
                Wijzigingen opslaan
            </button>
        </form>
    </div>
{/if}

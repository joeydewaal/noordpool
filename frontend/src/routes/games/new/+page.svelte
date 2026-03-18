<script lang="ts">
    import { goto } from "$app/navigation";
    import { auth } from "$lib/state/auth.svelte.ts";
    import { createGame } from "$lib/api/games.ts";
    import { createMutation, useQueryClient } from '@tanstack/svelte-query';
    import type { HomeAway, CreateGameRequest } from "$lib/api/types.ts";

    const canManage = $derived(auth.isAdmin || auth.isModerator);
    const queryClient = useQueryClient();

    let opponent = $state("");
    let location = $state("");
    let dateTime = $state("");
    let homeAway: HomeAway = $state("home");

    const createMut = createMutation(() => ({
        mutationFn: (data: CreateGameRequest) => createGame(data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['games'] });
            goto("/games");
        },
    }));

    function handleSubmit(e: Event) {
        e.preventDefault();
        createMut.mutate({
            opponent,
            location,
            dateTime: new Date(dateTime),
            homeAway,
        });
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
        <form
            onsubmit={handleSubmit}
            class="card p-6 space-y-4"
        >
            <div>
                <label for="opponent" class="label-text">Tegenstander</label>
                <input
                    id="opponent"
                    type="text"
                    bind:value={opponent}
                    required
                    class="input"
                />
            </div>
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
            <fieldset>
                <legend class="label-text mb-2">Thuis / Uit</legend>
                <div class="flex gap-4">
                    <label class="flex items-center gap-2">
                        <input
                            type="radio"
                            bind:group={homeAway}
                            value="home"
                            class="radio"
                        />
                        <span class="text-sm">Thuis</span>
                    </label>
                    <label class="flex items-center gap-2">
                        <input
                            type="radio"
                            bind:group={homeAway}
                            value="away"
                            class="radio"
                        />
                        <span class="text-sm">Uit</span>
                    </label>
                </div>
            </fieldset>
            <button
                type="submit"
                class="btn w-full preset-filled-primary-500"
            >
                Wedstrijd aanmaken
            </button>
        </form>
    </div>
{:else}
    <p class="text-error-500 font-medium">
        Geen toegang. Admin- of moderatorrol vereist.
    </p>
{/if}

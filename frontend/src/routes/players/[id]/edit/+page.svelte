<script lang="ts">
    import { page } from "$app/state";
    import { goto } from "$app/navigation";
    import { auth } from "$lib/state/auth.svelte.ts";
    import { getPlayer, updatePlayer } from "$lib/api/players.ts";
    import { createQuery, createMutation, useQueryClient } from '@tanstack/svelte-query';
    import type { Position, UpdatePlayerRequest } from "$lib/api/types.ts";

    const canManage = $derived(auth.isAdmin || auth.isModerator);
    const id = page.params.id;
    const queryClient = useQueryClient();

    const playerQuery = createQuery({
        queryKey: ['players', id],
        queryFn: () => getPlayer(id),
    });

    const updateMutation = createMutation({
        mutationFn: (data: UpdatePlayerRequest) => updatePlayer(id, data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['players'] });
            goto(`/players/${id}`);
        },
    });

    let name = $state("");
    let shirtNumber = $state(0);
    let position: Position = $state("midfielder");
    let active = $state(true);
    let initialized = $state(false);

    $effect(() => {
        const player = playerQuery.data;
        if (player && !initialized) {
            name = player.name;
            shirtNumber = player.shirtNumber;
            position = player.position;
            active = player.active;
            initialized = true;
        }
    });

    function handleSubmit(e: Event) {
        e.preventDefault();
        updateMutation.mutate({ name, shirtNumber, position, active });
    }
</script>

{#if !canManage}
    <p class="text-error-500 font-medium">
        Geen toegang. Admin- of moderatorrol vereist.
    </p>
{:else if playerQuery.isPending}
    <p class="text-surface-400">Laden...</p>
{:else if !playerQuery.data}
    <p class="text-surface-400">Speler niet gevonden.</p>
{:else if initialized}
    <div class="max-w-lg">
        <a
            href="/players/{id}"
            class="text-sm text-primary-500 hover:underline mb-4 inline-block"
            >&larr; Terug naar speler</a
        >
        <h1 class="text-2xl font-bold mb-6">Speler bewerken</h1>
        <form
            onsubmit={handleSubmit}
            class="card p-6 space-y-4"
        >
            <div>
                <label for="name" class="label-text">Naam</label>
                <input
                    id="name"
                    type="text"
                    bind:value={name}
                    required
                    class="input"
                />
            </div>
            <div>
                <label for="shirtNumber" class="label-text">Rugnummer</label>
                <input
                    id="shirtNumber"
                    type="number"
                    bind:value={shirtNumber}
                    min="1"
                    max="99"
                    required
                    class="input"
                />
            </div>
            <div>
                <label for="position" class="label-text">Positie</label>
                <select
                    id="position"
                    bind:value={position}
                    class="select"
                >
                    <option value="goalkeeper">Keeper</option>
                    <option value="defender">Verdediger</option>
                    <option value="midfielder">Middenvelder</option>
                    <option value="forward">Aanvaller</option>
                </select>
            </div>
            <div class="flex items-center gap-2">
                <input
                    id="active"
                    type="checkbox"
                    bind:checked={active}
                    class="checkbox"
                />
                <label for="active" class="text-sm font-medium">Actief</label>
            </div>
            <button
                type="submit"
                class="btn w-full preset-filled-primary-500"
            >
                Wijzigingen opslaan
            </button>
        </form>
    </div>
{/if}

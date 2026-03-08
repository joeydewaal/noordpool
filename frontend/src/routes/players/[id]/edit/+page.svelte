<script lang="ts">
    import { onMount } from "svelte";
    import { page } from "$app/state";
    import { goto } from "$app/navigation";
    import { auth } from "$lib/state/auth.svelte.js";
    import { getPlayer, updatePlayer } from "$lib/api/players.js";
    import type { Position } from "$lib/api/types.js";

    const canManage = $derived(auth.isAdmin || auth.isModerator);

    let name = $state("");
    let shirtNumber = $state(0);
    let position: Position = $state("midfielder");
    let active = $state(true);
    let loaded = $state(false);

    function handleSubmit(e: Event) {
        e.preventDefault();
        updatePlayer(page.params.id || "", {
            name,
            shirtNumber,
            position,
            active,
        });
        goto(`/players/${page.params.id}`);
    }

    onMount(() => {
        const player = getPlayer(page.params.id || "");
        if (player) {
            name = player.name;
            shirtNumber = player.shirtNumber;
            position = player.position;
            active = player.active;
            loaded = true;
        }
    });
</script>

{#if !canManage}
    <p class="text-red-600 font-medium">
        Geen toegang. Admin- of moderatorrol vereist.
    </p>
{:else if !loaded}
    <p class="text-gray-500">Speler niet gevonden.</p>
{:else}
    <div class="max-w-lg">
        <a
            href="/players/{page.params.id}"
            class="text-sm text-primary hover:underline mb-4 inline-block"
            >&larr; Terug naar speler</a
        >
        <h1 class="text-2xl font-bold text-gray-900 mb-6">Speler bewerken</h1>
        <form
            onsubmit={handleSubmit}
            class="bg-white rounded-lg shadow p-6 space-y-4"
        >
            <div>
                <label
                    for="name"
                    class="block text-sm font-medium text-gray-700 mb-1"
                    >Naam</label
                >
                <input
                    id="name"
                    type="text"
                    bind:value={name}
                    required
                    class="w-full border border-gray-300 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-primary/50"
                />
            </div>
            <div>
                <label
                    for="shirtNumber"
                    class="block text-sm font-medium text-gray-700 mb-1"
                    >Rugnummer</label
                >
                <input
                    id="shirtNumber"
                    type="number"
                    bind:value={shirtNumber}
                    min="1"
                    max="99"
                    required
                    class="w-full border border-gray-300 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-primary/50"
                />
            </div>
            <div>
                <label
                    for="position"
                    class="block text-sm font-medium text-gray-700 mb-1"
                    >Positie</label
                >
                <select
                    id="position"
                    bind:value={position}
                    class="w-full border border-gray-300 rounded-lg px-3 py-2 focus:outline-none focus:ring-2 focus:ring-primary/50"
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
                    class="rounded"
                />
                <label for="active" class="text-sm font-medium text-gray-700"
                    >Actief</label
                >
            </div>
            <button
                type="submit"
                class="w-full bg-primary hover:bg-primary-light text-white font-medium py-2.5 rounded-lg transition-colors"
            >
                Wijzigingen opslaan
            </button>
        </form>
    </div>
{/if}

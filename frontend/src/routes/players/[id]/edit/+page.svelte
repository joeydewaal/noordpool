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

    async function handleSubmit(e: Event) {
        e.preventDefault();
        await updatePlayer(page.params.id || "", {
            name,
            shirtNumber,
            position,
            active,
        });
        goto(`/players/${page.params.id}`);
    }

    onMount(async () => {
        const player = await getPlayer(page.params.id || "");
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
    <p class="text-error-500 font-medium">
        Geen toegang. Admin- of moderatorrol vereist.
    </p>
{:else if !loaded}
    <p class="text-surface-400">Speler niet gevonden.</p>
{:else}
    <div class="max-w-lg">
        <a
            href="/players/{page.params.id}"
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

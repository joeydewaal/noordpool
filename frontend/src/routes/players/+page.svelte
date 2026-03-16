<script lang="ts">
    import { onMount } from "svelte";
    import { auth } from "$lib/state/auth.svelte.ts";
    import { getPlayers } from "$lib/api/players.ts";
    import type { Player } from "$lib/api/types.ts";

    let players: Player[] = $state([]);
    let showInactive = $state(false);

    const canManage = $derived(auth.isAdmin || auth.isModerator);

    const filtered = $derived(
        showInactive ? players : players.filter((p) => p.active),
    );

    const positionColor: Record<string, string> = {
        goalkeeper: "preset-filled-warning-500",
        defender: "preset-filled-secondary-500",
        midfielder: "preset-filled-primary-500",
        forward: "preset-filled-error-500",
    };

    onMount(async () => {
        players = await getPlayers();
    });
</script>

<div class="flex items-center justify-between mb-6">
    <h1 class="text-2xl font-bold">Spelers</h1>
    <div class="flex items-center gap-3">
        {#if canManage}
            <label class="flex items-center gap-2 text-sm text-surface-400">
                <input
                    type="checkbox"
                    bind:checked={showInactive}
                    class="checkbox"
                />
                Toon inactief
            </label>
            <a
                href="/players/new"
                class="btn btn-sm preset-filled-primary-500"
            >
                Speler toevoegen
            </a>
        {/if}
    </div>
</div>

<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
    {#each filtered as player}
        <a
            href="/players/{player.id}"
            class="card preset-tonal-surface p-5 flex items-center gap-4 hover:preset-tonal-primary transition-colors {!player.active
                ? 'opacity-60'
                : ''}"
        >
            <div class="text-2xl font-bold text-primary-500 w-12 text-center">
                {player.shirtNumber}
            </div>
            <div class="flex-1 min-w-0">
                <div class="font-semibold truncate">
                    {player.name}
                </div>
                <span
                    class="chip mt-1 {positionColor[player.position]}"
                >
                    {player.position}
                </span>
                {#if !player.active}
                    <span
                        class="chip mt-1 ml-1 preset-tonal-surface"
                    >
                        inactief
                    </span>
                {/if}
            </div>
        </a>
    {/each}
</div>

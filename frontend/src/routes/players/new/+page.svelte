<script lang="ts">
    import { goto } from "$app/navigation";
    import { auth } from "$lib/state/auth.svelte";
    import { createPlayer } from "$lib/api/players";
    import { createMutation, useQueryClient } from "@tanstack/svelte-query";
    import type { Position, CreatePlayerRequest } from "$lib/api/types";

    const canManage = $derived(auth.isAdmin || auth.isModerator);
    const queryClient = useQueryClient();

    let firstName = $state("");
    let lastName = $state("");
    let shirtNumber = $state(0);
    let position: Position = $state("Centrale middenvelder");

    const createMut = createMutation(() => ({
        mutationFn: (data: CreatePlayerRequest) => createPlayer(data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ["players"] });
            goto("/players");
        },
    }));

    function handleSubmit(e: Event) {
        e.preventDefault();
        createMut.mutate({ firstName, lastName, shirtNumber, position });
    }
</script>

{#if canManage}
    <div class="max-w-lg">
        <button
            onclick={() => history.back()}
            class="text-sm text-primary-500 hover:underline mb-4 inline-block"
            >&larr; Alle spelers</button
        >
        <h1 class="text-2xl font-bold mb-6">Speler toevoegen</h1>
        <form onsubmit={handleSubmit} class="card p-6 space-y-4">
            <div>
                <label for="firstName" class="label-text">Voornaam</label>
                <input
                    id="firstName"
                    type="text"
                    bind:value={firstName}
                    required
                    class="input"
                />
            </div>
            <div>
                <label for="lastName" class="label-text">Achternaam</label>
                <input
                    id="lastName"
                    type="text"
                    bind:value={lastName}
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
                <select id="position" bind:value={position} class="select">
                    <option value="Keeper">Keeper</option>
                    <option value="Centrale verdediger"
                        >Centrale verdediger</option
                    >
                    <option value="Linksback">Linksback</option>
                    <option value="Rechtsback">Rechtsback</option>
                    <option value="Defensieve middenvelder"
                        >Defensieve middenvelder</option
                    >
                    <option value="Centrale middenvelder"
                        >Centrale middenvelder</option
                    >
                    <option value="Aanvallende middenvelder"
                        >Aanvallende middenvelder</option
                    >
                    <option value="Linksvleugel">Linksvleugel</option>
                    <option value="Rechtsvleugel">Rechtsvleugel</option>
                    <option value="Spits">Spits</option>
                </select>
            </div>
            <button type="submit" class="btn w-full preset-filled-primary-500">
                Speler aanmaken
            </button>
        </form>
    </div>
{:else}
    <p class="text-error-500 font-medium">
        Geen toegang. Admin- of moderatorrol vereist.
    </p>
{/if}

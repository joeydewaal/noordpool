<script lang="ts">
    import { page } from "$app/state";
    import { auth } from "$lib/state/auth.svelte";
    import { getPlayer, updatePlayer } from "$lib/api/players";
    import { updateUser } from "$lib/api/users";
    import { createQuery, createMutation, useQueryClient } from '@tanstack/svelte-query';
    import type { Position, UpdatePlayerRequest } from "$lib/api/types";

    const canManage = $derived(auth.isAdmin || auth.isModerator);
    const id = page.params.id!;
    const queryClient = useQueryClient();

    const playerQuery = createQuery(() => ({
        queryKey: ['players', id],
        queryFn: () => getPlayer(id),
    }));

    const updateMutation = createMutation(() => ({
        mutationFn: (data: UpdatePlayerRequest) => updatePlayer(id, data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['players'] });
            history.back();
        },
    }));

    const moderatorMutation = createMutation(() => ({
        mutationFn: ({ userId, isModerator }: { userId: string; isModerator: boolean }) =>
            updateUser(userId, { isModerator }),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['players', id] });
        },
    }));

    function toggleModerator() {
        const linkedUser = playerQuery.data?.user;
        if (!linkedUser) return;
        moderatorMutation.mutate({
            userId: linkedUser.id,
            isModerator: !linkedUser.isModerator,
        });
    }

    let firstName = $state("");
    let lastName = $state("");
    let shirtNumber = $state(0);
    let position: Position = $state("Centrale middenvelder");
    let active = $state(true);
    let initialized = $state(false);

    $effect(() => {
        const player = playerQuery.data;
        if (player && !initialized) {
            firstName = player.firstName;
            lastName = player.lastName;
            shirtNumber = player.shirtNumber;
            position = player.position;
            active = player.active;
            initialized = true;
        }
    });

    function handleSubmit(e: Event) {
        e.preventDefault();
        updateMutation.mutate({ firstName, lastName, shirtNumber, position, active });
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
        <button
            onclick={() => history.back()}
            class="text-sm text-primary-500 hover:underline mb-4 inline-block"
            >&larr; Terug naar speler</button
        >
        <h1 class="text-2xl font-bold mb-6">Speler bewerken</h1>
        <form
            onsubmit={handleSubmit}
            class="card p-6 space-y-4"
        >
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
                <select
                    id="position"
                    bind:value={position}
                    class="select"
                >
                    <option value="Keeper">Keeper</option>
                    <option value="Centrale verdediger">Centrale verdediger</option>
                    <option value="Linksback">Linksback</option>
                    <option value="Rechtsback">Rechtsback</option>
                    <option value="Defensieve middenvelder">Defensieve middenvelder</option>
                    <option value="Centrale middenvelder">Centrale middenvelder</option>
                    <option value="Aanvallende middenvelder">Aanvallende middenvelder</option>
                    <option value="Linksvleugel">Linksvleugel</option>
                    <option value="Rechtsvleugel">Rechtsvleugel</option>
                    <option value="Spits">Spits</option>
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

        {#if auth.isAdmin && playerQuery.data.user && !playerQuery.data.user.isAdmin}
            <div class="card p-6 mt-4">
                <h2 class="text-lg font-bold mb-2">Rol</h2>
                <p class="text-sm text-surface-400 mb-4">
                    {#if playerQuery.data.user.isModerator}
                        {firstName} {lastName} is moderator.
                    {:else}
                        {firstName} {lastName} heeft geen moderatorrol.
                    {/if}
                </p>
                <button
                    type="button"
                    onclick={toggleModerator}
                    disabled={moderatorMutation.isPending}
                    class="btn {playerQuery.data.user.isModerator
                        ? 'preset-tonal-tertiary'
                        : 'preset-filled-tertiary-500'}"
                    aria-label={playerQuery.data.user.isModerator
                        ? `Demote ${firstName} ${lastName}`
                        : `Promote ${firstName} ${lastName}`}
                >
                    {playerQuery.data.user.isModerator
                        ? "Verwijder moderator"
                        : "Maak moderator"}
                </button>
            </div>
        {/if}
    </div>
{/if}

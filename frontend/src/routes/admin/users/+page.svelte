<script lang="ts">
  import {
    createQuery,
    createMutation,
    useQueryClient,
  } from "@tanstack/svelte-query";
  import { auth } from "$lib/state/auth.svelte";
  import { listUsers, updateUser } from "$lib/api/users";
  import type { User } from "$lib/api/types";
  import Spinner from "$lib/components/Spinner.svelte";

  const queryClient = useQueryClient();

  const usersQuery = createQuery(() => ({
    queryKey: ["users"],
    queryFn: listUsers,
    enabled: auth.isAdmin,
  }));

  const toggleModeratorMut = createMutation(() => ({
    mutationFn: ({ id, isModerator }: { id: string; isModerator: boolean }) =>
      updateUser(id, { isModerator }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["users"] });
    },
  }));

  function toggleModerator(user: User) {
    toggleModeratorMut.mutate({
      id: user.id,
      isModerator: !user.isModerator,
    });
  }
</script>

{#if !auth.isAdmin}
  <p class="text-surface-400 text-sm">
    Geen toegang. Alleen beheerders kunnen deze pagina bekijken.
  </p>
{:else}
  <div class="flex items-center justify-between mb-6">
    <h1 class="text-2xl font-bold">Gebruikers</h1>
  </div>

  {#if usersQuery.isPending}
    <Spinner />
  {:else if usersQuery.isError}
    <p class="text-red-500 text-sm">Kon gebruikers niet laden</p>
  {:else if !usersQuery.data || usersQuery.data.length === 0}
    <p class="text-surface-400 text-sm">Geen gebruikers gevonden.</p>
  {:else}
    <div class="space-y-3">
      {#each usersQuery.data as user}
        <div
          class="card preset-tonal-surface p-4 flex items-center justify-between gap-4 flex-wrap"
        >
          <div class="flex-1 min-w-0">
            <div class="font-semibold truncate">
              {user.firstName}
              {user.lastName}
            </div>
            <div class="text-sm text-surface-400 truncate">{user.email}</div>
            <div class="flex flex-wrap gap-1 mt-2">
              {#each user.roles as role}
                <span
                  class="chip text-xs {role === 'admin'
                    ? 'preset-filled-warning-500'
                    : role === 'moderator'
                      ? 'preset-filled-primary-500'
                      : 'preset-tonal-surface'}"
                >
                  {role === "admin"
                    ? "Beheerder"
                    : role === "moderator"
                      ? "Moderator"
                      : "Speler"}
                </span>
              {/each}
              {#if user.roles.length === 0}
                <span class="chip text-xs preset-tonal-surface">Geen rol</span>
              {/if}
            </div>
          </div>
          <div class="flex items-center gap-2">
            {#if user.isAdmin}
              <span class="text-xs text-surface-400 italic">
                Beheerder — niet bewerkbaar
              </span>
            {:else}
              <button
                type="button"
                onclick={() => toggleModerator(user)}
                disabled={toggleModeratorMut.isPending}
                class="btn btn-sm {user.isModerator
                  ? 'preset-filled-primary-500'
                  : 'preset-tonal-surface'}"
              >
                {user.isModerator ? "Moderator" : "Maak moderator"}
              </button>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}
{/if}

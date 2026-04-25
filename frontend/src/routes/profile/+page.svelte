<script lang="ts">
  import { auth } from "$lib/state/auth.svelte";
  import { pwa } from "$lib/state/pwa.svelte";
  import { theme } from "$lib/state/theme.svelte";
  import { logout, unlinkPlayer } from "$lib/api/auth";
  import { deleteAvatar, uploadAvatar } from "$lib/api/users";
  import { getPlayer, updatePlayer } from "$lib/api/players";
  import { broadcastPush } from "$lib/api/push";
  import { goto } from "$app/navigation";
  import {
    createQuery,
    createMutation,
    useQueryClient,
  } from "@tanstack/svelte-query";
  import type { Position, UpdatePlayerRequest } from "$lib/api/types";
  import {
    disablePush,
    enablePush,
    isCurrentBrowserSubscribed,
    isPushSupported,
  } from "$lib/push-subscribe";
  import { onMount } from "svelte";
  import Spinner from "$lib/components/Spinner.svelte";

  if (!auth.isAuthenticated) {
    goto("/auth/login");
  }

  let hasPlayer = $derived(auth.user?.roles.includes("player") ?? false);
  const playerId = $derived(auth.playerId);
  const queryClient = useQueryClient();

  const playerQuery = createQuery(() => ({
    queryKey: ["players", playerId],
    queryFn: () => getPlayer(playerId!),
    enabled: !!playerId,
  }));

  const updateMutation = createMutation(() => ({
    mutationFn: (data: UpdatePlayerRequest) => updatePlayer(playerId!, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["players", playerId] });
    },
  }));

  let avatarInput = $state<HTMLInputElement | null>(null);
  let avatarBusy = $state(false);
  let avatarError = $state<string | null>(null);

  async function handleAvatarFile(e: Event) {
    const input = e.currentTarget as HTMLInputElement;
    const file = input.files?.[0];
    if (!file) return;
    avatarBusy = true;
    avatarError = null;
    try {
      const user = await uploadAvatar(file);
      auth.setUser(user);
    } catch (err: any) {
      avatarError =
        err?.response?.data?.error ??
        (err instanceof Error ? err.message : "Uploaden mislukt.");
    } finally {
      avatarBusy = false;
      input.value = "";
    }
  }

  async function handleAvatarDelete() {
    avatarBusy = true;
    avatarError = null;
    try {
      const user = await deleteAvatar();
      auth.setUser(user);
    } catch (err: any) {
      avatarError =
        err?.response?.data?.error ??
        (err instanceof Error ? err.message : "Verwijderen mislukt.");
    } finally {
      avatarBusy = false;
    }
  }

  let editingPlayer = $state(false);
  let shirtNumber = $state(0);
  let position: Position = $state("Centrale middenvelder");

  function startEditing() {
    const player = playerQuery.data;
    if (player) {
      shirtNumber = player.shirtNumber;
      position = player.position;
      editingPlayer = true;
    }
  }

  function cancelEditing() {
    editingPlayer = false;
  }

  function handlePlayerSubmit(e: Event) {
    e.preventDefault();
    updateMutation.mutate({ shirtNumber, position });
    editingPlayer = false;
  }

  async function handleUnlink() {
    const res = await unlinkPlayer();
    auth.setUser(res.user);
  }

  function handleLogout() {
    logout();
    auth.clear();
    goto("/");
  }

  // We track three pieces of state: whether the browser supports Web Push at
  // all, whether this device currently has an active subscription on the
  // server, and whether a toggle is in flight. Permission state ('denied') is
  // surfaced separately so we can show a help message instead of a button
  // that will silently fail.
  let pushSupported = $state(false);
  let pushSubscribed = $state(false);
  let pushPermission = $state<NotificationPermission>("default");
  let pushBusy = $state(false);
  let pushError = $state<string | null>(null);

  async function refreshPushState() {
    if (!isPushSupported()) {
      pushSupported = false;
      return;
    }
    pushSupported = true;
    pushPermission = Notification.permission;
    pushSubscribed = await isCurrentBrowserSubscribed();
  }

  onMount(() => {
    refreshPushState();
  });

  async function handleEnablePush() {
    pushBusy = true;
    pushError = null;
    try {
      await enablePush();
      await refreshPushState();
    } catch (err) {
      pushError = err instanceof Error ? err.message : "Inschakelen mislukt.";
      await refreshPushState();
    } finally {
      pushBusy = false;
    }
  }

  async function handleDisablePush() {
    pushBusy = true;
    pushError = null;
    try {
      await disablePush();
      await refreshPushState();
    } catch (err) {
      pushError = err instanceof Error ? err.message : "Uitschakelen mislukt.";
    } finally {
      pushBusy = false;
    }
  }

  let broadcastMessage = $state("");
  let broadcastBusy = $state(false);
  let broadcastError = $state<string | null>(null);
  let broadcastSent = $state(false);

  async function handleBroadcast() {
    if (!broadcastMessage.trim()) return;
    broadcastBusy = true;
    broadcastError = null;
    broadcastSent = false;
    try {
      await broadcastPush(broadcastMessage.trim());
      broadcastSent = true;
      broadcastMessage = "";
    } catch (err) {
      broadcastError =
        err instanceof Error ? err.message : "Versturen mislukt.";
    } finally {
      broadcastBusy = false;
    }
  }
</script>

{#if auth.isAuthenticated}
  <div class="max-w-md mx-auto p-6 space-y-6">
    <div class="card p-6 flex flex-col items-center gap-4">
      <div class="relative">
        {#if auth.user?.avatarUrl}
          <img
            src={auth.user.avatarUrl}
            alt="{auth.user.firstName} {auth.user.lastName}"
            class="w-20 h-20 rounded-full object-cover"
          />
        {:else}
          <div
            class="w-20 h-20 rounded-full bg-surface-500 flex items-center justify-center text-3xl font-bold text-white"
          >
            {auth.user?.firstName?.charAt(0).toUpperCase()}
          </div>
        {/if}
        {#if avatarBusy}
          <div
            class="absolute inset-0 flex items-center justify-center rounded-full bg-black/40"
          >
            <Spinner size="sm" />
          </div>
        {/if}
      </div>

      <input
        type="file"
        accept="image/jpeg,image/png,image/webp"
        class="hidden"
        bind:this={avatarInput}
        onchange={handleAvatarFile}
      />
      <div class="flex gap-2">
        <button
          type="button"
          class="btn btn-sm preset-filled-primary-500"
          onclick={() => avatarInput?.click()}
          disabled={avatarBusy}
        >
          Foto wijzigen
        </button>
        {#if auth.user?.avatarUrl}
          <button
            type="button"
            class="btn btn-sm preset-outlined-surface-500"
            onclick={handleAvatarDelete}
            disabled={avatarBusy}
          >
            Verwijderen
          </button>
        {/if}
      </div>
      {#if avatarError}
        <p class="text-xs text-error-500">{avatarError}</p>
      {/if}

      <div class="text-center">
        <h1 class="text-xl font-bold">
          {auth.user?.firstName}
          {auth.user?.lastName}
        </h1>
        <p class="text-surface-400">{auth.user?.email}</p>
      </div>

      {#if auth.user?.roles.length}
        <div class="flex flex-wrap gap-2 justify-center">
          {#each auth.user.roles as role}
            <span class="chip preset-filled-surface-500">{role}</span>
          {/each}
        </div>
      {/if}
    </div>

    {#if hasPlayer && playerId}
      <div class="card p-4 space-y-4">
        <div class="flex items-center justify-between">
          <span class="font-medium">Gekoppelde speler</span>
          <div class="flex gap-2">
            {#if !editingPlayer && playerQuery.data}
              <button
                class="btn btn-sm preset-filled-primary-500"
                onclick={startEditing}
              >
                Bewerken
              </button>
            {/if}
            <button
              class="btn btn-sm preset-filled-warning-500"
              onclick={handleUnlink}
            >
              Ontkoppelen
            </button>
          </div>
        </div>

        {#if playerQuery.isPending}
          <Spinner size="sm" />
        {:else if playerQuery.isError}
          <p class="text-error-500 text-sm">Kon speler niet laden</p>
        {:else if playerQuery.data}
          {#if editingPlayer}
            <form
              onsubmit={handlePlayerSubmit}
              class="space-y-3 pt-2 border-t border-surface-200 dark:border-surface-800"
            >
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
              <div class="flex gap-2">
                <button
                  type="submit"
                  class="btn btn-sm preset-filled-primary-500">Opslaan</button
                >
                <button
                  type="button"
                  class="btn btn-sm preset-outlined-surface-500"
                  onclick={cancelEditing}>Annuleren</button
                >
              </div>
            </form>
          {:else}
            <div
              class="flex items-center gap-3 pt-2 border-t border-surface-200 dark:border-surface-800"
            >
              <span class="text-2xl font-bold text-primary-500"
                >{playerQuery.data.shirtNumber}</span
              >
              <div>
                <p class="font-medium">
                  {playerQuery.data.firstName}
                  {playerQuery.data.lastName}
                </p>
                <p class="text-sm text-surface-400">
                  {playerQuery.data.position}
                </p>
              </div>
            </div>
          {/if}
        {/if}
      </div>
    {:else if !hasPlayer}
      <a
        href="/auth/link-player"
        class="card p-4 flex items-center justify-between"
      >
        <span class="font-medium">Geen speler gekoppeld</span>
        <span class="btn btn-sm preset-filled-primary-500">Koppelen</span>
      </a>
    {/if}

    <div class="card p-4 flex items-center justify-between">
      <span class="font-medium">Thema</span>
      <button
        class="btn preset-filled-surface-500"
        onclick={() => theme.toggle()}
      >
        {#if theme.isDark}
          <svg
            class="w-5 h-5"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              d="M12 3v1m0 16v1m8.66-13.66l-.71.71M4.05 19.95l-.71.71M21 12h-1M4 12H3m16.66 7.66l-.71-.71M4.05 4.05l-.71-.71M16 12a4 4 0 11-8 0 4 4 0 018 0z"
            />
          </svg>
          <span>Licht thema</span>
        {:else}
          <svg
            class="w-5 h-5"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              d="M21 12.79A9 9 0 1111.21 3a7 7 0 009.79 9.79z"
            />
          </svg>
          <span>Donker thema</span>
        {/if}
      </button>
    </div>

    <div class="card p-4 space-y-3">
      <div class="flex items-center justify-between">
        <div>
          <p class="font-medium">Meldingen</p>
          <p class="text-sm text-surface-400">
            Ontvang een melding bij elk doelpunt tijdens een live wedstrijd.
          </p>
        </div>
        {#if !pushSupported}
          <span class="chip preset-filled-surface-500">Niet ondersteund</span>
        {:else if pushPermission === "denied"}
          <span class="chip preset-filled-error-500">Geblokkeerd</span>
        {:else if pushSubscribed}
          <button
            class="btn btn-sm preset-filled-warning-500"
            onclick={handleDisablePush}
            disabled={pushBusy}
          >
            Uitschakelen
          </button>
        {:else}
          <button
            class="btn btn-sm preset-filled-primary-500"
            onclick={handleEnablePush}
            disabled={pushBusy}
          >
            Inschakelen
          </button>
        {/if}
      </div>
      {#if pushPermission === "denied"}
        <p class="text-xs text-surface-400">
          Meldingen zijn geblokkeerd voor deze site. Sta meldingen toe via de
          instellingen van je browser om ze opnieuw in te schakelen.
        </p>
      {/if}
      {#if pushError}
        <p class="text-xs text-error-500">{pushError}</p>
      {/if}
    </div>

    {#if auth.isAdmin}
      <div class="card p-4 space-y-3">
        <p class="font-medium">Melding versturen</p>
        <p class="text-sm text-surface-400">
          Stuur een pushmelding naar alle abonnees.
        </p>
        <form
          onsubmit={(e) => {
            e.preventDefault();
            handleBroadcast();
          }}
          class="flex gap-2"
        >
          <input
            type="text"
            bind:value={broadcastMessage}
            placeholder="Bericht..."
            class="input flex-1"
            disabled={broadcastBusy}
          />
          <button
            type="submit"
            class="btn btn-sm preset-filled-primary-500"
            disabled={broadcastBusy || !broadcastMessage.trim()}
          >
            Versturen
          </button>
        </form>
        {#if broadcastSent}
          <p class="text-xs text-success-500">Melding verstuurd.</p>
        {/if}
        {#if broadcastError}
          <p class="text-xs text-error-500">{broadcastError}</p>
        {/if}
      </div>
    {/if}

    {#if pwa.installable}
      <button
        class="btn preset-filled-surface-500 w-full md:hidden"
        onclick={() => pwa.install()}
      >
        <svg
          class="w-5 h-5"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            d="M4 16v2a2 2 0 002 2h12a2 2 0 002-2v-2M7 10l5 5 5-5M12 15V3"
          />
        </svg>
        <span>App installeren</span>
      </button>
    {/if}

    <button class="btn preset-filled-error-500 w-full" onclick={handleLogout}>
      <svg
        class="w-5 h-5"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        viewBox="0 0 24 24"
      >
        <path
          stroke-linecap="round"
          stroke-linejoin="round"
          d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1"
        />
      </svg>
      <span>Uitloggen</span>
    </button>
  </div>
{/if}

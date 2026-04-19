<script lang="ts">
  import { page } from "$app/state";
  import { auth } from "$lib/state/auth.svelte";
  import {
    createQuery,
    createMutation,
    useQueryClient,
  } from "@tanstack/svelte-query";
  import { getGame } from "$lib/api/games";
  import { getPlayers } from "$lib/api/players";
  import { getLineup, saveLineup } from "$lib/api/lineup";
  import { FORMATIONS } from "$lib/lineup/formations";
  import type { Formation } from "$lib/lineup/formations";
  import type { Player } from "$lib/api/types";
  import PitchView from "$lib/components/PitchView.svelte";
  import type { SlotData } from "$lib/components/PitchView.svelte";
  import Spinner from "$lib/components/Spinner.svelte";
  import { Dialog } from "@skeletonlabs/skeleton-svelte";
  import { toaster } from "$lib/state/toaster";

  const gameId = page.params.id!;
  const queryClient = useQueryClient();
  const canManage = $derived(auth.isAdmin || auth.isModerator);

  const gameQuery = createQuery(() => ({
    queryKey: ["games", gameId],
    queryFn: () => getGame(gameId),
  }));

  const lineupQuery = createQuery(() => ({
    queryKey: ["lineup", gameId],
    queryFn: () => getLineup(gameId),
  }));

  const playersQuery = createQuery(() => ({
    queryKey: ["players"],
    queryFn: getPlayers,
    enabled: canManage,
  }));

  const saveMutation = createMutation(() => ({
    mutationFn: (data: Parameters<typeof saveLineup>[1]) =>
      saveLineup(gameId, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["lineup", gameId] });
      editMode = false;
      toaster.success({ title: "Opstelling opgeslagen" });
    },
  }));

  let editMode = $state(false);
  let draftFormation = $state<Formation>("4-4-2");
  let draftSlots = $state<Map<number, string>>(new Map()); // slot → playerId
  let draftCaptain = $state<string | null>(null);

  type DialogMode = "picker" | "menu";
  let dialogOpen = $state(false);
  let dialogMode = $state<DialogMode>("picker");
  let activeSlot = $state<number | null>(null);

  const activePlayers = $derived(
    (playersQuery.data ?? []).filter((p) => p.active),
  );
  const assignedIds = $derived(new Set(draftSlots.values()));
  const unassigned = $derived(
    activePlayers.filter((p) => !assignedIds.has(p.id)),
  );

  function startEdit() {
    const l = lineupQuery.data;
    if (l) {
      draftFormation = l.formation;
      const m = new Map<number, string>();
      for (const s of l.slots) m.set(s.slot, s.player.id);
      draftSlots = m;
      draftCaptain = l.slots.find((s) => s.captain)?.player.id ?? null;
    } else {
      draftFormation = "4-4-2";
      draftSlots = new Map();
      draftCaptain = null;
    }
    editMode = true;
  }

  function cancelEdit() {
    editMode = false;
    dialogOpen = false;
    activeSlot = null;
  }

  function handleSave() {
    const slots = [...draftSlots.entries()].map(([slot, playerId]) => ({
      slot,
      playerId,
      captain: playerId === draftCaptain,
    }));
    saveMutation.mutate({ formation: draftFormation, slots });
  }

  function handleSlotClick(slotIdx: number) {
    activeSlot = slotIdx;
    dialogMode = draftSlots.has(slotIdx) ? "menu" : "picker";
    dialogOpen = true;
  }

  function assignPlayer(playerId: string) {
    if (activeSlot === null) return;
    draftSlots = new Map(draftSlots).set(activeSlot, playerId);
    dialogOpen = false;
  }

  function clearSlot() {
    if (activeSlot === null) return;
    const pid = draftSlots.get(activeSlot);
    if (pid === draftCaptain) draftCaptain = null;
    const m = new Map(draftSlots);
    m.delete(activeSlot);
    draftSlots = m;
    dialogOpen = false;
  }

  function toggleCaptain(playerId: string) {
    draftCaptain = draftCaptain === playerId ? null : playerId;
    dialogOpen = false;
  }

  function playerById(id: string): Player | undefined {
    return activePlayers.find((p) => p.id === id);
  }

  function buildSlotData(slotIdx: number): SlotData | null {
    if (editMode) {
      const pid = draftSlots.get(slotIdx);
      if (!pid) return null;
      const p = playerById(pid);
      if (!p) return null;
      return {
        lastName: p.lastName,
        shirtNumber: p.shirtNumber,
        avatarUrl: p.user?.avatarUrl ?? null,
        captain: pid === draftCaptain,
      };
    }
    const s = lineupQuery.data?.slots.find((s) => s.slot === slotIdx);
    if (!s) return null;
    return {
      lastName: s.player.lastName,
      shirtNumber: s.player.shirtNumber,
      avatarUrl: s.player.avatarUrl,
      captain: s.captain,
    };
  }

  const pitchSlots = $derived(
    Array.from({ length: 11 }, (_, i) => buildSlotData(i)),
  );

  const benchSlots = $derived(
    Array.from({ length: 7 }, (_, i) => buildSlotData(11 + i)),
  );

  const activeMenuInfo = $derived(
    activeSlot !== null ? buildSlotData(activeSlot) : null,
  );

  const currentFormation = $derived<Formation>(
    editMode ? draftFormation : (lineupQuery.data?.formation ?? "4-4-2"),
  );
</script>

<div class="max-w-lg">
  <button
    onclick={() => history.back()}
    class="text-sm text-primary-500 hover:underline mb-4 inline-block"
  >
    &larr; Terug
  </button>

  {#if gameQuery.data}
    {@const g = gameQuery.data}
    <div class="mb-3 flex items-center justify-between">
      <div>
        <h1 class="text-lg font-bold">Opstelling</h1>
        <p class="text-sm text-surface-400">
          {g.homeTeam.name} – {g.awayTeam.name}
        </p>
      </div>
      {#if canManage}
        {#if editMode}
          <div class="flex gap-2">
            <button
              type="button"
              class="btn btn-sm preset-outlined-surface-500"
              onclick={cancelEdit}
            >
              Annuleren
            </button>
            <button
              type="button"
              class="btn btn-sm preset-filled-primary-500"
              disabled={saveMutation.isPending}
              onclick={handleSave}
            >
              {#if saveMutation.isPending}<Spinner
                  size="sm"
                />{:else}Opslaan{/if}
            </button>
          </div>
        {:else}
          <button
            type="button"
            class="btn btn-sm preset-outlined-warning-500"
            onclick={startEdit}
          >
            Bewerken
          </button>
        {/if}
      {/if}
    </div>
  {/if}

  {#if lineupQuery.isPending}
    <Spinner />
  {:else}
    <!-- Formation selector (edit mode) -->
    {#if editMode}
      <div class="flex flex-wrap gap-1.5 mb-4">
        {#each FORMATIONS as f}
          <button
            type="button"
            class="btn btn-sm {draftFormation === f.name
              ? 'preset-filled-primary-500'
              : 'preset-outlined-surface-500'}"
            onclick={() => (draftFormation = f.name)}
          >
            {f.name}
          </button>
        {/each}
      </div>
    {:else if lineupQuery.data}
      <div class="mb-3">
        <span class="chip preset-tonal-surface text-sm font-semibold">
          {lineupQuery.data.formation}
        </span>
      </div>
    {/if}

    {#if !editMode && !lineupQuery.data}
      <div class="card p-6 text-center text-surface-400">
        <p>Geen opstelling gepubliceerd.</p>
        {#if canManage}
          <button
            type="button"
            class="btn btn-sm preset-outlined-primary-500 mt-3"
            onclick={startEdit}
          >
            Opstelling aanmaken
          </button>
        {/if}
      </div>
    {:else}
      <PitchView
        formation={currentFormation}
        slots={pitchSlots}
        bench={benchSlots}
        {editMode}
        onSlotClick={editMode ? handleSlotClick : undefined}
      />
    {/if}
  {/if}
</div>

<!-- Picker / menu dialog -->
<Dialog
  open={dialogOpen}
  onOpenChange={(e) => {
    dialogOpen = e.open;
    if (!e.open) activeSlot = null;
  }}
>
  <Dialog.Backdrop class="fixed inset-0 bg-black/50 z-40" />
  <Dialog.Positioner
    class="fixed inset-0 flex items-end justify-center z-50 p-4 sm:items-center"
  >
    <Dialog.Content
      class="card bg-surface-100-900 p-5 w-full max-w-sm shadow-xl"
      aria-label={dialogMode === "picker" ? "Speler kiezen" : "Slot opties"}
    >
      {#if dialogMode === "menu" && activeMenuInfo}
        <Dialog.Title class="text-base font-bold mb-4">
          {activeMenuInfo.lastName} #{activeMenuInfo.shirtNumber}
        </Dialog.Title>
        <div class="flex flex-col gap-2">
          <button
            type="button"
            class="btn preset-outlined-surface-500 justify-start"
            onclick={() => {
              const pid =
                activeSlot !== null ? draftSlots.get(activeSlot) : null;
              if (pid) toggleCaptain(pid);
            }}
          >
            {#if activeSlot !== null && draftSlots.get(activeSlot) === draftCaptain}
              ✓ Aanvoerder
            {:else}
              Aanvoerder instellen
            {/if}
          </button>
          <button
            type="button"
            class="btn preset-outlined-surface-500 justify-start"
            onclick={() => (dialogMode = "picker")}
          >
            Speler wisselen
          </button>
          <button
            type="button"
            class="btn preset-outlined-error-500 justify-start"
            onclick={clearSlot}
          >
            Verwijderen
          </button>
        </div>
      {:else}
        <Dialog.Title class="text-base font-bold mb-3"
          >Speler kiezen</Dialog.Title
        >
        {#if unassigned.length === 0}
          <p class="text-sm text-surface-400">
            Alle spelers zijn al ingedeeld.
          </p>
        {:else}
          <div class="flex flex-col gap-1 max-h-72 overflow-y-auto pr-1">
            {#each unassigned as p (p.id)}
              <button
                type="button"
                class="btn btn-sm preset-outlined-surface-500 flex items-center gap-3 justify-start"
                onclick={() => assignPlayer(p.id)}
              >
                <span
                  class="w-7 h-7 rounded-full bg-neutral-600 flex items-center justify-center text-xs font-bold text-white shrink-0"
                >
                  {p.shirtNumber}
                </span>
                <span class="text-sm">{p.firstName} {p.lastName}</span>
              </button>
            {/each}
          </div>
        {/if}
      {/if}
    </Dialog.Content>
  </Dialog.Positioner>
</Dialog>

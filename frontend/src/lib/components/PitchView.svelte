<script lang="ts">
  import { getFormation } from "$lib/lineup/formations";
  import type { Formation } from "$lib/lineup/formations";
  import PlayerBadge from "./PlayerBadge.svelte";

  export interface SlotData {
    lastName: string;
    shirtNumber: number;
    avatarUrl: string | null;
    captain: boolean;
  }

  interface Props {
    formation: Formation;
    /** slot index → player data, null = empty */
    slots: (SlotData | null)[];
    /** bench slots 11–17 (7 entries) */
    bench: (SlotData | null)[];
    editMode?: boolean;
    onSlotClick?: (slotIdx: number) => void;
  }

  let {
    formation,
    slots,
    bench,
    editMode = false,
    onSlotClick,
  }: Props = $props();

  const formationDef = $derived(getFormation(formation));
</script>

<!-- Half-pitch (bottom half): GK at bottom, forwards near centre line at top.
     Portrait aspect kept for mobile legibility; markings scaled accordingly. -->
<div
  class="relative w-full rounded-lg overflow-hidden"
  style="aspect-ratio: 2/3; background: linear-gradient(to bottom, #19421a 0%, #143514 100%);"
>
  <!-- Pitch markings (SVG viewBox matches 2:3 container ratio) -->
  <svg
    viewBox="0 0 100 150"
    class="absolute inset-0 w-full h-full pointer-events-none"
    style="opacity: 0.28;"
    aria-hidden="true"
  >
    <!-- Half-pitch boundary -->
    <rect
      x="3"
      y="3"
      width="94"
      height="144"
      fill="none"
      stroke="white"
      stroke-width="1.2"
    />
    <!-- Centre line at top -->
    <line x1="3" y1="3" x2="97" y2="3" stroke="white" stroke-width="1.5" />
    <!-- Centre circle arc (bottom half visible) -->
    <path
      d="M 37 3 A 13 9 0 0 1 63 3"
      fill="none"
      stroke="white"
      stroke-width="0.8"
    />
    <!-- Penalty box -->
    <rect
      x="20"
      y="102"
      width="60"
      height="45"
      fill="none"
      stroke="white"
      stroke-width="0.8"
    />
    <!-- Goal area -->
    <rect
      x="37"
      y="132"
      width="26"
      height="15"
      fill="none"
      stroke="white"
      stroke-width="0.6"
    />
    <!-- Penalty spot -->
    <circle cx="50" cy="117" r="1.2" fill="white" />
    <!-- Goal frame -->
    <rect
      x="44"
      y="147"
      width="12"
      height="4"
      fill="none"
      stroke="white"
      stroke-width="0.8"
    />
  </svg>

  <!-- Outfield + GK slots (0–10) -->
  {#each Array.from({ length: 11 }, (_, i) => i) as slotIdx}
    {@const pos = formationDef.slots[slotIdx]}
    {@const info = slots[slotIdx] ?? null}
    <div
      class="absolute"
      style="left: {pos.x}%; top: {pos.y}%; transform: translate(-50%, -50%);"
    >
      {#if info}
        <PlayerBadge
          lastName={info.lastName}
          shirtNumber={info.shirtNumber}
          avatarUrl={info.avatarUrl}
          captain={info.captain}
          size="sm"
          onclick={editMode && onSlotClick
            ? () => onSlotClick(slotIdx)
            : undefined}
        />
      {:else if editMode && onSlotClick}
        <button
          type="button"
          class="flex items-center justify-center text-white/40 hover:text-white/70 hover:border-white/50 transition-colors border-2 border-dashed border-white/25 rounded"
          style="width: 66px; height: 90px;"
          onclick={() => onSlotClick(slotIdx)}
          aria-label="Speler toevoegen"
        >
          <span class="text-xl leading-none">+</span>
        </button>
      {/if}
    </div>
  {/each}
</div>

<!-- Bench (slots 11–17) -->
<div class="mt-3">
  <p
    class="text-xs font-semibold text-surface-400 uppercase tracking-wide mb-2"
  >
    Bank
  </p>
  <div class="flex gap-2 overflow-x-auto pb-1">
    {#each bench as info, i}
      {#if info}
        <PlayerBadge
          lastName={info.lastName}
          shirtNumber={info.shirtNumber}
          avatarUrl={info.avatarUrl}
          captain={info.captain}
          size="sm"
          onclick={editMode && onSlotClick
            ? () => onSlotClick(11 + i)
            : undefined}
        />
      {:else if editMode && onSlotClick}
        <button
          type="button"
          class="shrink-0 flex items-center justify-center text-surface-400/50 hover:text-surface-400 hover:border-surface-400/60 transition-colors border-2 border-dashed border-surface-400/30 rounded"
          style="width: 66px; height: 90px;"
          onclick={() => onSlotClick(11 + i)}
          aria-label="Wisselspeler toevoegen"
        >
          <span class="text-xl leading-none">+</span>
        </button>
      {:else}
        <div
          class="shrink-0 rounded border border-dashed border-surface-700/40"
          style="width: 66px; height: 90px;"
        ></div>
      {/if}
    {/each}
  </div>
</div>

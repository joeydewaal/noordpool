<script lang="ts">
  interface Props {
    avatarUrl?: string | null;
    shirtNumber: number;
    lastName: string;
    captain?: boolean;
    size?: "sm" | "md";
    onclick?: () => void;
  }

  let {
    avatarUrl,
    shirtNumber,
    lastName,
    captain = false,
    size = "sm",
    onclick,
  }: Props = $props();

  const cfg = $derived(
    (
      {
        sm: { w: 54, h: 72, imgSize: 32, numSize: 12, nameSize: 8 },
        md: { w: 68, h: 90, imgSize: 40, numSize: 14, nameSize: 10 },
      } as const
    )[size],
  );

  const displayName = $derived(
    lastName.length > 8 ? lastName.slice(0, 7) + "." : lastName,
  );
</script>

<!--
  FIFA-style card: pronounced angled cuts at the two top corners,
  image section (upper ~65%) + name bar (lower ~35%).
-->
<button
  type="button"
  class="relative flex flex-col select-none focus:outline-none transition-[filter]"
  class:cursor-pointer={!!onclick}
  class:cursor-default={!onclick}
  class:hover:brightness-125={!!onclick}
  style="
    width: {cfg.w}px;
    height: {cfg.h}px;
    clip-path: polygon(22% 0%, 78% 0%, 100% 18%, 100% 100%, 0% 100%, 0% 18%);
    background: linear-gradient(170deg, #3c3c3c 0%, #222 55%, #141414 100%);
  "
  {onclick}
>
  <!-- Sheen overlay — simulates foil gloss -->
  <div
    class="absolute inset-0 pointer-events-none"
    style="background: linear-gradient(155deg, rgba(255,255,255,0.11) 0%, rgba(255,255,255,0.03) 35%, transparent 65%);"
  ></div>

  {#if captain}
    <span
      class="absolute font-bold text-amber-400 z-10 leading-none"
      style="top: 22%; right: 9%; font-size: {Math.round(cfg.w * 0.13)}px;"
    >
      C
    </span>
  {/if}

  <!-- Image area fills top ~65% -->
  <div class="flex-1 flex items-center justify-center z-10 pt-1">
    {#if avatarUrl}
      <img
        src={avatarUrl}
        alt={lastName}
        class="rounded-full object-cover ring-1 ring-white/10"
        style="width: {cfg.imgSize}px; height: {cfg.imgSize}px;"
      />
    {:else}
      <div
        class="rounded-full bg-neutral-700 flex items-center justify-center font-bold text-white ring-1 ring-white/10"
        style="width: {cfg.imgSize}px; height: {cfg.imgSize}px; font-size: {cfg.numSize}px;"
      >
        {shirtNumber}
      </div>
    {/if}
  </div>

  <!-- Name bar at the bottom -->
  <div
    class="z-10 w-full shrink-0 flex items-center justify-center"
    style="
      height: 35%;
      background: rgba(0,0,0,0.4);
      border-top: 1px solid rgba(255,255,255,0.08);
      padding: 0 3px;
    "
  >
    <span
      class="font-bold text-white uppercase tracking-wide truncate leading-none text-center block w-full"
      style="font-size: {cfg.nameSize}px;"
    >
      {displayName}
    </span>
  </div>
</button>

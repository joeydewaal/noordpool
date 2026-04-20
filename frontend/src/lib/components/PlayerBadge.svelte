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
        sm: { w: 66, h: 90, imgSize: 34, numSize: 13, nameSize: 8 },
        md: { w: 84, h: 114, imgSize: 44, numSize: 16, nameSize: 10 },
      } as const
    )[size],
  );

  const displayName = $derived(
    lastName.length > 8 ? lastName.slice(0, 7) + "." : lastName,
  );
</script>

<!--
  FIFA-style shield card:
  - Shield silhouette: small beveled top corners + pointed bottom at 80%
  - Fully gold metallic gradient
  - Layered metallic rim via inset box-shadow
  - Player silhouette SVG as fallback when no avatar
-->
<button
  type="button"
  class="relative shrink-0 select-none focus:outline-none transition-[filter]"
  class:cursor-pointer={!!onclick}
  class:cursor-default={!onclick}
  class:hover:brightness-110={!!onclick}
  style="
    width: {cfg.w}px;
    height: {cfg.h}px;
    clip-path: polygon(0% 9%, 9% 0%, 91% 0%, 100% 9%, 100% 80%, 50% 100%, 0% 80%);
    background: linear-gradient(160deg, #f5d060 0%, #d4a020 38%, #a87010 68%, #d0a830 100%);
    box-shadow:
      inset 0 0 0 1.5px rgba(255,255,255,0.45),
      inset 0 0 0 3px rgba(100,50,0,0.22),
      0 4px 14px rgba(0,0,0,0.55);
  "
  {onclick}
>
  <!-- Depth gradient overlay -->
  <div
    class="absolute inset-0 pointer-events-none"
    style="background: linear-gradient(160deg, rgba(255,255,255,0.14) 0%, transparent 45%, rgba(0,0,0,0.16) 100%);"
  ></div>

  <!-- Diagonal gloss sheen -->
  <div
    class="absolute inset-0 pointer-events-none"
    style="background: linear-gradient(130deg, rgba(255,255,255,0.24) 0%, rgba(255,255,255,0.06) 28%, transparent 52%);"
  ></div>

  <!-- Shirt number — top-left (like FIFA rating) -->
  <span
    class="absolute z-10 font-black leading-none"
    style="top: 10%; left: 10%; font-size: {cfg.numSize}px; color: #2c1400;"
    >{shirtNumber}</span
  >

  <!-- Captain marker — top-right -->
  {#if captain}
    <span
      class="absolute z-10 font-black leading-none"
      style="top: 10%; right: 11%; font-size: {cfg.numSize -
        1}px; color: #2c1400;">C</span
    >
  {/if}

  <!-- Avatar / silhouette — upper center -->
  <div
    class="absolute z-10"
    style="top: 14%; left: 50%; transform: translateX(-50%);"
  >
    {#if avatarUrl}
      <img
        src={avatarUrl}
        alt={lastName}
        class="rounded-full object-cover"
        style="width: {cfg.imgSize}px; height: {cfg.imgSize}px; box-shadow: 0 2px 6px rgba(0,0,0,0.35), 0 0 0 1.5px rgba(255,255,255,0.3);"
      />
    {:else}
      <!-- Player silhouette outline -->
      <div
        class="flex items-end justify-center"
        style="width: {cfg.imgSize}px; height: {cfg.imgSize}px;"
      >
        <svg
          viewBox="0 0 24 24"
          fill="currentColor"
          style="width: 88%; height: 88%; color: rgba(44,20,0,0.38);"
          aria-hidden="true"
        >
          <path
            d="M12 12c2.21 0 4-1.79 4-4s-1.79-4-4-4-4 1.79-4 4 1.79 4 4 4zm0 2c-2.67 0-8 1.34-8 4v2h16v-2c0-2.66-5.33-4-8-4z"
          />
        </svg>
      </div>
    {/if}
  </div>

  <!-- Name strip — just above the taper (bottom 22%) -->
  <div
    class="absolute z-10 left-0 right-0 flex items-center justify-center"
    style="
      bottom: 22%;
      height: 18%;
      background: rgba(0,0,0,0.3);
      border-top: 1px solid rgba(0,0,0,0.18);
      padding: 0 4px;
    "
  >
    <span
      class="font-extrabold uppercase block w-full text-center leading-none truncate"
      style="font-size: {cfg.nameSize}px; letter-spacing: 0.05em; color: #fff5cc;"
      >{displayName}</span
    >
  </div>
</button>

<style>
  @media (max-width: 640px) {
    button {
      transform: scale(0.82);
    }
  }
</style>

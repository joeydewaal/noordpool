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
        sm: {
          w: 54,
          h: 76,
          imgSize: 26,
          numSize: 11,
          nameSize: 7,
          nameBottom: "21%",
        },
        md: {
          w: 68,
          h: 96,
          imgSize: 34,
          numSize: 14,
          nameSize: 9,
          nameBottom: "21%",
        },
      } as const
    )[size],
  );

  const displayName = $derived(
    lastName.length > 8 ? lastName.slice(0, 7) + "." : lastName,
  );
</script>

<!--
  FIFA-style shield card:
  - Shield silhouette: beveled top corners + pointed bottom (80% → 100% taper)
  - Entirely gold: warm metallic gradient throughout
  - Layered metallic rim via inset box-shadow
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
      inset 0 0 0 1.5px rgba(255,255,255,0.4),
      inset 0 0 0 3px rgba(120,60,0,0.25),
      0 4px 12px rgba(0,0,0,0.55);
  "
  {onclick}
>
  <!-- Inner gold gradient for depth -->
  <div
    class="absolute inset-0 pointer-events-none"
    style="background: linear-gradient(160deg, rgba(255,255,255,0.12) 0%, transparent 45%, rgba(0,0,0,0.18) 100%);"
  ></div>

  <!-- Diagonal gloss sheen -->
  <div
    class="absolute inset-0 pointer-events-none"
    style="background: linear-gradient(130deg, rgba(255,255,255,0.22) 0%, rgba(255,255,255,0.06) 30%, transparent 55%);"
  ></div>

  <!-- Shirt number — top-left, dark for contrast -->
  <span
    class="absolute z-10 font-black leading-none"
    style="top: 10%; left: 10%; font-size: {cfg.numSize}px; color: #2c1600; text-shadow: 0 1px 0 rgba(255,220,80,0.4);"
    >{shirtNumber}</span
  >

  <!-- Captain marker — top-right -->
  {#if captain}
    <span
      class="absolute z-10 font-black leading-none"
      style="top: 10%; right: 11%; font-size: {cfg.numSize -
        1}px; color: #2c1600;">C</span
    >
  {/if}

  <!-- Avatar / initials circle — upper center -->
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
      <div
        class="rounded-full flex items-center justify-center font-black"
        style="
          width: {cfg.imgSize}px;
          height: {cfg.imgSize}px;
          font-size: {Math.round(cfg.imgSize * 0.38)}px;
          color: #2c1600;
          background: rgba(0,0,0,0.18);
          box-shadow: inset 0 1px 3px rgba(0,0,0,0.3), 0 0 0 1.5px rgba(255,255,255,0.25);
        "
      >
        {shirtNumber}
      </div>
    {/if}
  </div>

  <!-- Name strip — dark semi-transparent bar, placed in the flat rectangle area above the taper -->
  <div
    class="absolute z-10 left-0 right-0 flex items-center justify-center"
    style="
      bottom: {cfg.nameBottom};
      height: 18%;
      background: rgba(0,0,0,0.32);
      border-top: 1px solid rgba(0,0,0,0.2);
      padding: 0 4px;
    "
  >
    <span
      class="font-extrabold uppercase block w-full text-center leading-none truncate"
      style="font-size: {cfg.nameSize}px; letter-spacing: 0.05em; color: #fff8e0;"
      >{displayName}</span
    >
  </div>
</button>

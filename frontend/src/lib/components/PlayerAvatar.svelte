<script lang="ts">
  interface Props {
    avatarUrl?: string | null;
    shirtNumber: number;
    name?: string;
    size?: "sm" | "md" | "lg";
  }

  let { avatarUrl, shirtNumber, name, size = "md" }: Props = $props();

  const dims = {
    sm: "w-10 h-10 text-lg",
    md: "w-12 h-12 text-2xl",
    lg: "w-16 h-16 text-4xl",
  };
</script>

{#if avatarUrl}
  <img
    src={avatarUrl}
    alt={name ?? `#${shirtNumber}`}
    class="{dims[size]} rounded-full object-cover shrink-0"
  />
{:else}
  <div
    class="{dims[
      size
    ]} rounded-full flex items-center justify-center font-bold text-primary-500 shrink-0"
  >
    {shirtNumber}
  </div>
{/if}

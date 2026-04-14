<script lang="ts">
  type Size = "sm" | "md" | "lg";
  interface Props {
    size?: Size;
    label?: string;
    center?: boolean;
  }
  let { size = "md", label = "Laden...", center = true }: Props = $props();

  const sizeClass = $derived(
    {
      sm: "w-4 h-4 border-2",
      md: "w-8 h-8 border-[3px]",
      lg: "w-12 h-12 border-4",
    }[size],
  );
</script>

<div
  class={center ? "flex justify-center py-6" : "inline-flex"}
  role="status"
  aria-live="polite"
>
  <span
    class="{sizeClass} inline-block rounded-full border-surface-700 border-t-primary-500 animate-spin"
  ></span>
  <span class="sr-only">{label}</span>
</div>

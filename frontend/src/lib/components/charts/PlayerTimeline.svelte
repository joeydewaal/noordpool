<script lang="ts">
  import { LineChart } from "layerchart";
  import type { GameTimelineEntry } from "$lib/api/types";

  interface Props {
    timeline: GameTimelineEntry[];
  }

  let { timeline }: Props = $props();

  const data = $derived(
    timeline.map((entry) => ({
      ...entry,
      date: new Date(entry.dateTime),
    })),
  );
</script>

{#if data.length >= 2}
  <div class="h-72">
    <LineChart
      {data}
      x="date"
      y="cumulativeGoals"
      series={[
        {
          key: "goals",
          label: "Doelpunten",
          value: "cumulativeGoals",
          color: "oklch(0.67 0.2 150)",
        },
        {
          key: "assists",
          label: "Assists",
          value: "cumulativeAssists",
          color: "oklch(0.57 0.14 175)",
        },
      ]}
      padding={{ left: 30, top: 4, right: 16, bottom: 36 }}
      points={true}
      legend={true}
      rule={false}
      props={{
        spline: { class: "stroke-2" },
      }}
    />
  </div>
{:else}
  <p class="text-surface-400 text-sm text-center py-8">
    Niet genoeg data om een grafiek te tonen.
  </p>
{/if}

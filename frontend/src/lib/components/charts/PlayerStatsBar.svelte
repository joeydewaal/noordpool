<script lang="ts">
  import { BarChart } from "layerchart";

  interface Props {
    goals: number;
    assists: number;
    yellowCards: number;
    redCards: number;
  }

  let { goals, assists, yellowCards, redCards }: Props = $props();

  const data = $derived([
    { category: "Doelpunten", value: goals },
    { category: "Assists", value: assists },
    { category: "Gele kaarten", value: yellowCards },
    { category: "Rode kaarten", value: redCards },
  ]);
</script>

<div class="h-72">
  <BarChart
    {data}
    x="value"
    y="category"
    orientation="horizontal"
    series={[{ key: "default", value: "value" }]}
    c="category"
    cRange={[
      "oklch(0.67 0.2 150)",
      "oklch(0.57 0.14 175)",
      "oklch(0.82 0.14 77)",
      "oklch(0.64 0.22 29)",
    ]}
    bandPadding={0.5}
    padding={{ left: 100, top: 4, right: 40, bottom: 4 }}
    grid={false}
    rule={false}
    axis="y"
    labels={{ format: (value: number) => Math.round(value).toString() }}
    tooltip={false}
  />
</div>

<script lang="ts">
	import { LineChart } from 'layerchart';
	import type { GameTimelineEntry } from '$lib/api/types';

	interface Props {
		timeline: GameTimelineEntry[];
	}

	let { timeline }: Props = $props();

	const data = $derived(
		timeline.map((entry) => ({
			...entry,
			date: new Date(entry.dateTime),
		}))
	);
</script>

{#if data.length >= 2}
	<div class="h-64">
		<LineChart
			{data}
			x="date"
			y="cumulativeGoals"
			series={[
				{ key: 'goals', label: 'Doelpunten', value: 'cumulativeGoals', color: '#22c55e' },
				{ key: 'assists', label: 'Assists', value: 'cumulativeAssists', color: '#3b82f6' },
			]}
			points={true}
			legend={true}
		/>
	</div>
{:else}
	<p class="text-surface-400 text-sm text-center py-8">Niet genoeg data om een grafiek te tonen.</p>
{/if}

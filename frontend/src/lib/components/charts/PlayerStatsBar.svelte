<script lang="ts">
	import { BarChart } from 'layerchart';

	interface Props {
		goals: number;
		assists: number;
		yellowCards: number;
		redCards: number;
	}

	let { goals, assists, yellowCards, redCards }: Props = $props();

	const data = $derived([
		{ category: 'Doelpunten', value: goals },
		{ category: 'Assists', value: assists },
		{ category: 'Gele kaarten', value: yellowCards },
		{ category: 'Rode kaarten', value: redCards },
	]);
</script>

<div class="h-64">
	<BarChart
		{data}
		x="category"
		y="value"
		series={[
			{ key: 'default', value: 'value' }
		]}
		grid={false}
		rule={false}
	/>
</div>

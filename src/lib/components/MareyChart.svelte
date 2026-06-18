<script lang="ts">
	import { onMount } from 'svelte';
	import * as d3 from 'd3';

	let { mareyData } = $props<{
		mareyData: {
			route_id: string;
			route: string;
			stations: { name: string; crs: string; mileage: number; type: string }[];
			services: { id: string; operator: string; direction: string; days: string[]; stops: { station: string; arr: number | null; dep: number | null }[] }[];
		} | null;
	}>();

	let container = $state<HTMLDivElement | null>(null);
	let activeDay = $state('MF');
	let tooltip = $state<{ visible: boolean; x: number; y: number; content: string }>({ visible: false, x: 0, y: 0, content: '' });

	onMount(() => {
		if (mareyData) renderChart();
	});

	$effect(() => {
		if (mareyData) renderChart();
	});

	function renderChart() {
		if (!container || !mareyData) return;
		const data = mareyData;

		// Filter services by day
		let services = data.services.filter((s) => s.days.includes(activeDay));
		if (services.length === 0) services = data.services.slice(0, 50); // fallback

		const stations = data.stations;
		const margin = { top: 40, right: 40, bottom: 60, left: 120 };
		const width = Math.max(800, stations.length * 30) - margin.left - margin.right;
		const height = 600 - margin.top - margin.bottom;

		// Clear previous
		d3.select(container).selectAll('*').remove();

		const svg = d3.select(container)
			.append('svg')
			.attr('viewBox', `0 0 ${width + margin.left + margin.right} ${height + margin.top + margin.bottom}`)
			.append('g')
			.attr('transform', `translate(${margin.left},${margin.top})`);

		// Scales
		const maxMileage = d3.max(stations, (d) => d.mileage) || 100;
		const maxTime = 1440; // 24 hours in minutes

		const yScale = d3.scaleLinear().domain([0, maxMileage]).range([height, 0]);
		const xScale = d3.scaleLinear().domain([0, maxTime]).range([0, width]);

		// Station labels (Y-axis)
		svg.selectAll('.station-label')
			.data(stations)
			.enter()
			.append('text')
			.attr('class', 'station-label')
			.attr('x', -10)
			.attr('y', (d) => yScale(d.mileage))
			.attr('dy', '0.35em')
			.attr('text-anchor', 'end')
			.attr('fill', '#94a3b8')
			.attr('font-size', '11px')
			.text((d) => d.crs);

		// Station grid lines
		svg.selectAll('.station-line')
			.data(stations)
			.enter()
			.append('line')
			.attr('class', 'station-line')
			.attr('x1', 0)
			.attr('x2', width)
			.attr('y1', (d) => yScale(d.mileage))
			.attr('y2', (d) => yScale(d.mileage))
			.attr('stroke', '#334155')
			.attr('stroke-dasharray', '2,2');

		// Time axis (X-axis)
		const timeAxis = d3.axisBottom(xScale)
			.tickValues(d3.range(0, 1441, 120))
			.tickFormat((d: number) => `${Math.floor(d / 60)}:00`);

		svg.append('g')
			.attr('transform', `translate(0,${height})`)
			.call(timeAxis)
			.selectAll('text')
			.attr('fill', '#94a3b8');

		// Marey lines (one per service)
		const line = d3.line<{ x: number; y: number }>()
			.x((d) => xScale(d.x))
			.y((d) => yScale(d.y))
			.curve(d3.curveStepAfter);

		const serviceGroups = svg.selectAll('.service')
			.data(services.slice(0, 200)) // Limit for performance
			.enter()
			.append('g')
			.attr('class', 'service');

		serviceGroups.each(function(svc) {
			const g = d3.select(this);
			const points: { x: number; y: number }[] = [];

			for (const stop of svc.stops) {
				const station = stations.find((s) => s.crs === stop.station);
				if (!station) continue;
				const time = stop.dep ?? stop.arr;
				if (time === null) continue;
				points.push({ x: time, y: station.mileage });
			}

			if (points.length < 2) return;

			g.append('path')
				.datum(points)
				.attr('d', line)
				.attr('fill', 'none')
				.attr('stroke', '#60a5fa')
				.attr('stroke-width', 1.5)
				.attr('opacity', 0.7)
				.on('mouseover', function() {
					d3.select(this).attr('stroke', '#f59e0b').attr('stroke-width', 3).attr('opacity', 1);
				})
				.on('mouseout', function() {
					d3.select(this).attr('stroke', '#60a5fa').attr('stroke-width', 1.5).attr('opacity', 0.7);
				})
				.on('mousemove', (event) => {
					tooltip = {
						visible: true,
						x: event.pageX + 10,
						y: event.pageY - 10,
						content: `${svc.id} (${svc.operator})`
					};
				});
		});

		// Title
		svg.append('text')
			.attr('x', width / 2)
			.attr('y', -15)
			.attr('text-anchor', 'middle')
			.attr('fill', '#e2e8f0')
			.attr('font-size', '14px')
			.attr('font-weight', 'bold')
			.text(`${data.route} — ${activeDay}`);
	}
</script>

<div class="max-w-6xl mx-auto px-4 py-8">
	{#if mareyData}
		<div class="mb-4 flex gap-2">
			{#each [...new Set(mareyData.services.flatMap(s => s.days))] as day}
				<button on:click={() => activeDay = day}
					class="px-3 py-1.5 rounded text-sm font-medium transition-colors {activeDay === day ? 'bg-blue-600 text-white' : 'bg-slate-800 text-slate-300 hover:bg-slate-700'}"
				>{day}</button>
			{/each}
		</div>

		<div bind:this={container} class="overflow-x-auto bg-slate-900 rounded-lg border border-slate-700 p-4"></div>

		{#if tooltip.visible}
			<div class="fixed z-50 bg-slate-800 border border-slate-600 rounded px-3 py-2 text-sm text-slate-200 shadow-lg"
				style="left: {tooltip.x}px; top: {tooltip.y}px;">
				{tooltip.content}
			</div>
		{/if}
	{:else}
		<div class="text-center py-12 text-slate-400">No Marey data available</div>
	{/if}
</div>

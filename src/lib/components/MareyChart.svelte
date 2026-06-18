<script lang="ts">
	import { onMount } from 'svelte';
	import * as d3 from 'd3';

	const OP_COLORS: Record<string, string> = {
		'CC': '#009E73', 'XC': '#009E73', 'SE': '#009E73', 'LE': '#009E73',
		'EM': '#CC79A7', 'GR': '#CC79A7', 'AW': '#CC79A7',
		'LO': '#E86A10', 'ME': '#E86A10',
		'VT': '#E32636', 'HX': '#E32636', 'HT': '#E32636',
		'GW': '#56B4E9', 'SR': '#56B4E9',
		'TP': '#D55E00', 'TL': '#D55E00', 'LM': '#D55E00',
		'NT': '#0072B2', 'SW': '#0072B2', 'CH': '#0072B2',
		'SN': '#F0E442', 'GN': '#F0E442',
		'GC': '#882255', 'GX': '#56B4E9', 'LF': '#E86A10',
		'XR': '#D55E00',
	};
	const DEFAULT_COLOR = '#64748b'; // slate-500 for unknown operators

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

	function getOperatorColor(op: string): string {
		return OP_COLORS[op] || DEFAULT_COLOR;
	}

	function renderChart() {
		if (!container || !mareyData) return;
		const data = mareyData;

		// Filter services by day
		let services = data.services.filter((s) => s.days.includes(activeDay));
		if (services.length === 0) services = data.services.slice(0, 50);

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

		// Pre-process: build points with midnight normalization, compute max time
		const maxMileage = d3.max(stations, (d) => d.mileage) || 100;
		let maxTime = 1440;
		const serviceLines: { points: { x: number; y: number }[]; color: string; svc: (typeof services)[0] }[] = [];

		for (const svc of services.slice(0, 200)) {
			const points: { x: number; y: number }[] = [];
			let prevTime = -1;
			let timeOffset = 0;
			let maxValidX = -1;

			for (const stop of svc.stops) {
				const station = stations.find((s) => s.crs === stop.station);
				if (!station) continue;
				const time = stop.dep ?? stop.arr;
				if (time === null) continue;

				// Detect midnight crossing: time drops from >=12h (evening) to <4h (early morning)
				if (prevTime >= 720 && time < 240) {
					timeOffset += 1440;
				}
				prevTime = time;

				const x = time + timeOffset;
				const y = station.mileage;

				// Skip points where time goes backward (data quality issue from parser
				// where arrival/departure column misalignment creates backward jumps)
				if (x < maxValidX) continue;
				maxValidX = x;

				if (x > maxTime) maxTime = x;
				points.push({ x, y });
			}

			if (points.length >= 2) {
				serviceLines.push({ points, color: getOperatorColor(svc.operator), svc });
			}
		}

		// Round max time to next hour for clean axis
		if (maxTime > 1440) {
			maxTime = Math.max(1440, Math.ceil(maxTime / 60) * 60);
		}

		// Scales
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

		// Midnight marker line
		if (maxTime > 1440) {
			const mx = xScale(1440);
			svg.append('line')
				.attr('x1', mx)
				.attr('x2', mx)
				.attr('y1', 0)
				.attr('y2', height)
				.attr('stroke', '#f59e0b')
				.attr('stroke-width', 1)
				.attr('stroke-dasharray', '4,4')
				.attr('opacity', 0.4);
			svg.append('text')
				.attr('x', mx)
				.attr('y', -5)
				.attr('text-anchor', 'middle')
				.attr('fill', '#f59e0b')
				.attr('font-size', '10px')
				.attr('opacity', 0.5)
				.text('midnight');
			// Shade the "next day" area
			svg.append('rect')
				.attr('x', mx)
				.attr('y', 0)
				.attr('width', width - mx)
				.attr('height', height)
				.attr('fill', '#1e3a5f')
				.attr('opacity', 0.08);
		}

		// Time axis (X-axis) — extends past midnight if needed
		const tickStep = maxTime > 1440 ? 240 : 120;
		const axisTicks: number[] = [];
		for (let t = 0; t <= maxTime; t += tickStep) {
			axisTicks.push(t);
		}

		const timeAxis = d3.axisBottom(xScale)
			.tickValues(axisTicks)
			.tickFormat((d: number) => {
				const h = Math.floor(d / 60);
				const m = d % 60;
				return `${h}:${m.toString().padStart(2, '0')}`;
			});

		svg.append('g')
			.attr('transform', `translate(0,${height})`)
			.call(timeAxis)
			.selectAll('text')
			.attr('fill', '#94a3b8');

		// Marey lines (one per service, coloured by operator)
		const line = d3.line<{ x: number; y: number }>()
			.x((d) => xScale(d.x))
			.y((d) => yScale(d.y))
			.curve(d3.curveStepAfter);

		for (const { points, color, svc } of serviceLines) {
			svg.append('path')
				.datum(points)
				.attr('d', line)
				.attr('fill', 'none')
				.attr('stroke', color)
				.attr('stroke-width', 1.5)
				.attr('opacity', 0.7)
				.on('mouseover', function () {
					d3.select(this).attr('stroke', '#fcd34d').attr('stroke-width', 3).attr('opacity', 1);
				})
				.on('mouseout', function () {
					d3.select(this).attr('stroke', color).attr('stroke-width', 1.5).attr('opacity', 0.7);
				})
				.on('mousemove', (event) => {
					tooltip = {
						visible: true,
						x: event.pageX + 10,
						y: event.pageY - 10,
						content: `${svc.id} (${svc.operator || '?'})`
					};
				});
		}

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

		<!-- Legend -->
		<div class="mt-4 flex flex-wrap gap-x-4 gap-y-1.5 text-xs text-slate-400">
			{#each [...new Set(mareyData.services.map(s => s.operator).filter(Boolean))].sort() as op}
				<span class="inline-flex items-center gap-1.5">
					<span class="inline-block w-3 h-0.5 rounded" style="background: {OP_COLORS[op] || DEFAULT_COLOR}"></span>
					{op}
				</span>
			{/each}
		</div>
	{:else}
		<div class="text-center py-12 text-slate-400">No Marey data available</div>
	{/if}
</div>

<style>
	@media print {
		.no-print { display: none !important; }
		.print-only { display: inline !important; }
		body { background: white !important; color: black !important; }
	}
	.print-only { display: none; }
</style>

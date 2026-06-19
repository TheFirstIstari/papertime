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
	// For within-operator differentiation: subtle hue shifts by service index
	const DEFAULT_COLOR = '#64748b'; // slate-500

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

	// Generate a slightly varied shade of the same hue for visual distinction
	function varyColor(baseColor: string, index: number, total: number): string {
		if (total <= 1) return baseColor;
		// Parse the hex color
		const r = parseInt(baseColor.slice(1, 3), 16);
		const g = parseInt(baseColor.slice(3, 5), 16);
		const b = parseInt(baseColor.slice(5, 7), 16);
		// Slightly adjust brightness based on index
		const factor = 0.85 + (index / Math.max(total - 1, 1)) * 0.3;
		const nr = Math.min(255, Math.max(0, Math.round(r * factor)));
		const ng = Math.min(255, Math.max(0, Math.round(g * factor)));
		const nb = Math.min(255, Math.max(0, Math.round(b * factor)));
		return `#${nr.toString(16).padStart(2, '0')}${ng.toString(16).padStart(2, '0')}${nb.toString(16).padStart(2, '0')}`;
	}

	function getServiceColor(op: string, svcId: string, svcIndex: number, totalWithSameOp: number): string {
		if (op && OP_COLORS[op]) {
			// Vary within operator group so services are distinguishable
			if (totalWithSameOp > 1) {
				return varyColor(OP_COLORS[op], svcIndex, totalWithSameOp);
			}
			return OP_COLORS[op];
		}
		// Unknown operator: use HSL spread across services
		const hue = (svcIndex * 137.5) % 360; // golden angle for even distribution
		return `hsl(${hue}, 65%, 55%)`;
	}

	function renderEmpty() {
		if (!container) return;
		d3.select(container).selectAll('*').remove();
		const svg = d3.select(container)
			.append('svg')
			.attr('viewBox', '0 0 400 200')
			.append('g')
			.attr('transform', 'translate(200,100)');
		svg.append('text')
			.attr('text-anchor', 'middle')
			.attr('dy', '0.35em')
			.attr('fill', '#64748b')
			.attr('font-size', '14px')
			.text('No services operate on this day');
	}

	function renderChart() {
		if (!container || !mareyData) return;
		const data = mareyData;

		// Filter services by day
		let services = data.services.filter((s) => s.days.includes(activeDay));
		if (services.length === 0) {
			renderEmpty();
			return;
		}

		// Build operator group indices for color variation
		const opGroups = new Map<string, number>();
		const opTotals = new Map<string, number>();
		for (const s of services) {
			const op = s.operator || '';
			opTotals.set(op, (opTotals.get(op) || 0) + 1);
		}

		const stations = data.stations;
		const margin = { top: 40, right: 40, bottom: 60, left: 140 };
		const minLabelSpacing = 16;
		const chartHeight = Math.max(500, stations.length * minLabelSpacing);
		const width = Math.max(800, stations.length * 30) - margin.left - margin.right;
		const height = chartHeight - margin.top - margin.bottom;

		// Clear previous
		d3.select(container).selectAll('*').remove();

		const svg = d3.select(container)
			.append('svg')
			.attr('viewBox', `0 0 ${width + margin.left + margin.right} ${height + margin.top + margin.bottom}`)
			.append('g')
			.attr('transform', `translate(${margin.left},${margin.top})`);

		// Pre-process: build points with midnight normalization
		const maxMileage = d3.max(stations, (d) => d.mileage) || 100;
		let maxTime = 1440;
		const lineServices: { points: { x: number; y: number }[]; color: string; svc: (typeof services)[0] }[] = [];
		const dotServices: { x: number; y: number; color: string; svc: (typeof services)[0] }[] = [];
		const stationMap = new Map(stations.map((s) => [s.crs, s]));

		for (const svc of services) {
			const op = svc.operator || '';
			const opIndex = opGroups.get(op) || 0;
			opGroups.set(op, opIndex + 1);

			const points: { x: number; y: number }[] = [];
			let prevTime = -1;
			let timeOffset = 0;
			let lastValidX = -1;

			for (const stop of svc.stops) {
				const station = stationMap.get(stop.station);
				if (!station) continue;
				const time = stop.dep ?? stop.arr;
				if (time === null) continue;

				// Robust midnight detection: if time jumps backward by >10 hours, treat as midnight crossing
				if (prevTime >= 0 && time < prevTime && prevTime - time > 600) {
					timeOffset += 1440;
				}
				prevTime = time;

				const x = time + timeOffset;
				const y = station.mileage;

				// Skip points where time goes backward (data quality issues)
				if (x <= lastValidX) continue;
				lastValidX = x;

				if (x > maxTime) maxTime = x;
				points.push({ x, y });
			}

			const color = getServiceColor(svc.operator, svc.id, opIndex, opTotals.get(op) || 1);
			if (points.length >= 2) {
				lineServices.push({ points, color, svc });
			} else if (points.length === 1) {
				dotServices.push({ ...points[0], color, svc });
			}
		}

		// Round max time up to next hour for clean axis
		maxTime = Math.max(1440, Math.ceil(maxTime / 60) * 60);

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
			.text((d) => d.name || d.crs);

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

		// Midnight marker line — only shown when chart extends past midnight
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
				.text('00:00');
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
				const label = `${h}:${m.toString().padStart(2, '0')}`;
				return label;
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
			.curve(d3.curveLinear);

		for (const { points, color, svc } of lineServices) {
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
						content: `${svc.id} (${svc.operator || svc.id.match(/^(\w+)/)?.[1] || '?'})`
					};
				});
		}

		// Dots for services with only 1 valid point
		for (const { x, y, color, svc } of dotServices) {
			svg.append('circle')
				.attr('cx', xScale(x))
				.attr('cy', yScale(y))
				.attr('r', 2.5)
				.attr('fill', color)
				.attr('opacity', 0.5)
				.on('mouseover', function () {
					d3.select(this).attr('r', 5).attr('opacity', 1);
				})
				.on('mouseout', function () {
					d3.select(this).attr('r', 2.5).attr('opacity', 0.5);
				})
				.on('mousemove', (event) => {
					tooltip = {
						visible: true,
						x: event.pageX + 10,
						y: event.pageY - 10,
						content: `${svc.id} (${svc.operator || svc.id.match(/^(\w+)/)?.[1] || '?'})`
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

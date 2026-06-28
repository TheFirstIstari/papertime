<script lang="ts">
	export const csr = true;
	import { onMount } from 'svelte';
	import { loadStationServices, loadMareyData } from '$lib/data';
	import type { StationServices, ServiceRef, CallRef, StationPattern } from '$lib/types';
	import type { MareyData, MareyStation, MareyService, MareyStop } from '$lib/types';
	import MareyChart from '$lib/components/MareyChart.svelte';
	import PatternDiagram from '$lib/components/PatternDiagram.svelte';

	let { data } = $props<{ data: { crs: string } }>();
	let crs = data.crs;
	let stationData: StationServices | null = null;
	let mareyData: MareyData | null = null;
	let patternData: StationPattern | null = null;
	let loading = $state(true);
	let patternLoading = $state(true);
	let error = $state('');
	let filterOperator = $state('');
	let filterDest = $state('');
	let activeTab = $state<'timetable' | 'marey' | 'pattern'>('timetable');

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

	function formatTime(val: number | string | null): string {
		if (val === null || val === undefined) return '---';
		let minutes: number;
		if (typeof val === 'string') {
			const parts = val.split(':');
			if (parts.length < 2) return '---';
			minutes = parseInt(parts[0], 10) * 60 + parseInt(parts[1], 10);
		} else {
			minutes = val;
		}
		if (isNaN(minutes)) return '---';
		const h = Math.floor(minutes / 60) % 24;
		const m = minutes % 60;
		return `${h.toString().padStart(2, '0')}:${m.toString().padStart(2, '0')}`;
	}

	function getDepTime(s: ServiceRef): string {
		const call = s.calls?.find(c => c.crs === crs);
		return formatTime(call?.dep ?? null);
	}

	function getArrTime(s: ServiceRef): string {
		const call = s.calls?.find(c => c.crs === crs);
		return formatTime(call?.arr ?? null);
	}

	let filteredServices = $derived(
		svcList.filter(s => {
			if (filterOperator && s.operator !== filterOperator) return false;
			if (filterDest && (s.destination_name || s.destination) !== filterDest) return false;
			return true;
		})
	);

	let operators = $derived([...new Set(stationData?.services?.map(s => s.operator) ?? [])].sort());
	let destinations = $derived([...new Set(stationData?.services?.map(s => s.destination_name) ?? [])].sort());

	onMount(async () => {
		try {
			stationData = await loadStationServices(crs);
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load station data';
		}

		// Load Marey data
		try {
			mareyData = await loadMareyData(crs);
		} catch (e) {
			// Marey data optional
		}

		// Load pattern data
		try {
			const patternResp = await fetch(`/patterns/${crs}.json`);
			if (patternResp.ok) {
				patternData = await patternResp.json();
				// Populate station names from station-index if available
				try {
					const idxResp = await fetch('/station-index.json');
					if (idxResp.ok) {
						const idxData = await idxResp.json();
						const nameMap = new Map(idxData.map((s: any) => [s.id, s.name]));
						for (const branch of patternData.branches) {
							if (branch.next_stop) {
								branch.next_stop_name = nameMap.get(branch.next_stop) || branch.next_stop_name;
							}
							for (const svc of branch.services) {
								// no-op, already has data
							}
						}
					}
				} catch (e) {
					// Ignore index load errors
				}
			}
		} catch (e) {
			// Pattern data optional
		}
		patternLoading = false;

		loading = false;
	});
</script>

<svelte:head>
	<title>{stationData?.name ?? crs} — PaperTime</title>
</svelte:head>

<div class="min-h-screen bg-slate-900 text-slate-100">
	<div class="max-w-6xl mx-auto px-4 py-8">
		<a href="/" class="text-blue-400 hover:text-blue-300 text-sm mb-4 inline-block">&larr; Back to search</a>

		{#if loading}
			<div class="text-center py-12 text-slate-400">Loading...</div>
		{:else if error}
			<div class="text-center py-12 text-red-400">{error}</div>
		{:else if stationData}
			<h1 class="text-3xl font-bold mb-2">{stationData.name || stationData.station || crs}</h1>
			{@const svcList = stationData.services || []}
			<p class="text-slate-400 mb-6">{svcList.length} services</p>

			<!-- Tab switcher -->
			<div class="flex gap-2 mb-6">
				<button
					on:click={() => activeTab = 'timetable'}
					class="px-4 py-2 rounded text-sm font-medium transition-colors {activeTab === 'timetable' ? 'bg-blue-600 text-white' : 'bg-slate-800 text-slate-300 hover:bg-slate-700'}"
				>Timetable</button>
				<button
					on:click={() => activeTab = 'marey'}
					class="px-4 py-2 rounded text-sm font-medium transition-colors {activeTab === 'marey' ? 'bg-blue-600 text-white' : 'bg-slate-800 text-slate-300 hover:bg-slate-700'}"
					disabled={!mareyData}
				>Marey Chart</button>
				<button
					on:click={() => activeTab = 'pattern'}
					class="px-4 py-2 rounded text-sm font-medium transition-colors {activeTab === 'pattern' ? 'bg-blue-600 text-white' : 'bg-slate-800 text-slate-300 hover:bg-slate-700'}"
				>Patterns</button>
			</div>

			{#if activeTab === 'timetable'}
				{#if operators.length > 1 || destinations.length > 1}
					<div class="flex gap-3 mb-6">
						{#if operators.length > 1}
							<select bind:value={filterOperator} class="bg-slate-800 border border-slate-700 rounded px-3 py-1.5 text-sm">
								<option value="">All operators</option>
								{#each operators as op}
									<option value={op}>{op}</option>
								{/each}
							</select>
						{/if}
						{#if destinations.length > 1}
							<select bind:value={filterDest} class="bg-slate-800 border border-slate-700 rounded px-3 py-1.5 text-sm">
								<option value="">All destinations</option>
								{#each destinations as dest}
									<option value={dest}>{dest}</option>
								{/each}
							</select>
						{/if}
					</div>
				{/if}

				<div class="overflow-x-auto">
					<table class="w-full text-sm">
						<thead>
							<tr class="text-left text-slate-400 border-b border-slate-700">
								<th class="pb-2 pr-4">Time</th>
								<th class="pb-2 pr-4">Headcode</th>
								<th class="pb-2 pr-4">Operator</th>
								<th class="pb-2 pr-4">Destination</th>
								<th class="pb-2 pr-4">Calling at</th>
							</tr>
						</thead>
						<tbody>
							{#each filteredServices as svc (svc.id || svc.headcode)}
								<tr class="border-b border-slate-800 hover:bg-slate-800/50">
									<td class="py-2 pr-4 font-mono">{getDepTime(svc)}</td>
									<td class="py-2 pr-4">{svc.headcode}</td>
									<td class="py-2 pr-4">
										<span class="px-1.5 py-0.5 rounded text-xs font-medium" style="color: {OP_COLORS[svc.operator] || '#64748b'}">
											{svc.operator}
										</span>
									</td>
									<td class="py-2 pr-4">{svc.destination_name || svc.destination || '—'}</td>
									<td class="py-2 pr-4 text-slate-400 text-xs">
										{(svc.calls || []).slice(1, -1).map((c: CallRef) => c.crs).join(', ')}
									</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>

				{#if filteredServices.length === 0}
					<div class="text-center py-8 text-slate-400">No services match your filters.</div>
				{/if}

			{:else if activeTab === 'marey' && mareyData}
				<MareyChart {mareyData} />
			{:else if activeTab === 'pattern'}
				<PatternDiagram {patternData} loading={patternLoading} error={!patternData && !patternLoading ? 'No pattern data available' : ''} />
			{/if}
			{/if}
			</div>
			</div>

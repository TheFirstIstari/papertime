<script lang="ts">
	export const csr = true;
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import Fuse from 'fuse.js';
	import { findMatchingTables, formatTime } from '$lib/search';
	import { loadStations, loadTableIndex, getGapCount } from '$lib/data';
	import type { StationEntry, TableEntry, TableMatch } from '$lib/types';

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
	const DEFAULT_OP_COLOR = '#64748b';

	let stations: StationEntry[] = [];
	let tableIndex: TableEntry[] = [];
	let fuse: Fuse<StationEntry> | null = null;
	let originQuery = $state('');
	let destQuery = $state('');
	let fromCrs = $state('');
	let toCrs = $state('');
	let originSuggestions: StationEntry[] = $state([]);
	let destSuggestions: StationEntry[] = $state([]);
	let matches: TableMatch[] = $state([]);
	let gapCount = $state(0);
	let loaded = $state(false);

	onMount(async () => {
		try {
			[stations, tableIndex] = await Promise.all([loadStations(), loadTableIndex()]);
			fuse = new Fuse(stations, { keys: ['name', 'id'], threshold: 0.3 });
			gapCount = getGapCount(tableIndex);
			loaded = true;
			const params = $page.url.searchParams;
			const from = params.get('from');
			const to = params.get('to');
			if (from && to) {
				fromCrs = from;
				toCrs = to;
				const fromStn = stations.find(s => s.id === from);
				const toStn = stations.find(s => s.id === to);
				originQuery = fromStn?.name ?? from;
				destQuery = toStn?.name ?? to;
				runSearch(fromCrs, toCrs);
			}
		} catch (err) {
			console.error('PaperTime init failed:', err);
		}
	});

	function onOriginInput() {
		if (!fuse) return;
		originSuggestions = originQuery.length > 0 ? fuse.search(originQuery).map((r) => r.item).slice(0, 8) : [];
	}
	function onDestInput() {
		if (!fuse) return;
		destSuggestions = destQuery.length > 0 ? fuse.search(destQuery).map((r) => r.item).slice(0, 8) : [];
	}
	function selectOrigin(s: StationEntry) { originQuery = s.name; originSuggestions = []; }
	function selectDest(s: StationEntry) { destQuery = s.name; destSuggestions = []; }
	function runSearch(from: string, to: string) { matches = findMatchingTables(from, to, stations, tableIndex); }
	function onSubmit() {
		const fromStation = stations.find((s) => s.name === originQuery || s.id === originQuery);
		const toStation = stations.find((s) => s.name === destQuery || s.id === destQuery);
		if (fromStation && toStation) {
			fromCrs = fromStation.id;
			toCrs = toStation.id;
			runSearch(fromCrs, toCrs);
			goto(`/?from=${fromCrs}&to=${toCrs}`);
		}
	}

	const popularRoutes = [
		{ from: 'EUS', to: 'BHM', label: 'London → Birmingham' },
		{ from: 'EUS', to: 'MAN', label: 'London → Manchester' },
		{ from: 'EUS', to: 'EDI', label: 'London → Edinburgh' },
		{ from: 'EUS', to: 'LIV', label: 'London → Liverpool' },
		{ from: 'EUS', to: 'GLA', label: 'London → Glasgow' },
		{ from: 'EUS', to: 'OXF', label: 'London → Oxford' },
		{ from: 'EUS', to: 'BRI', label: 'London → Bristol' },
		{ from: 'EUS', to: 'LEE', label: 'London → Leeds' },
	];
</script>

<svelte:head>
	<title>PaperTime — May 2026 National Rail Timetables</title>
	<meta name="description" content="Explore the May 2026 National Rail timetable — paper timetables, iBRY Marey graphs, and service pattern diagrams." />
</svelte:head>

<div class="min-h-screen bg-slate-900 text-slate-100">
	<div class="max-w-4xl mx-auto px-4 py-12">
		<div class="text-center mb-10">
			<h1 class="text-5xl font-bold tracking-tight">PaperTime</h1>
			<p class="text-xl text-slate-400 mt-2">May 2026 National Rail Timetables</p>
			<p class="text-sm text-slate-500 mt-1">Paper timetables · iBRY Marey graphs · Service pattern diagrams</p>
		</div>

		{#if loaded}
			<form on:submit|preventDefault={onSubmit} class="mb-8">
				<div class="flex flex-col sm:flex-row gap-3">
					<div class="relative flex-1">
						<input type="text" bind:value={originQuery} on:input={onOriginInput}
							placeholder="Origin station (e.g. London Euston)"
							class="w-full bg-slate-800 border border-slate-700 rounded-lg px-4 py-3 text-slate-100 placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500" />
						{#if originSuggestions.length > 0}
							<div class="absolute z-10 w-full mt-1 bg-slate-800 border border-slate-700 rounded-lg shadow-lg overflow-hidden">
								{#each originSuggestions as s}
									<button type="button" on:click={() => selectOrigin(s)} class="w-full text-left px-4 py-2 hover:bg-slate-700 transition-colors">
										<span class="font-medium">{s.name}</span>
										<span class="text-slate-500 ml-2 text-sm">({s.id})</span>
									</button>
								{/each}
							</div>
						{/if}
					</div>
					<div class="relative flex-1">
						<input type="text" bind:value={destQuery} on:input={onDestInput}
							placeholder="Destination station (e.g. Birmingham)"
							class="w-full bg-slate-800 border border-slate-700 rounded-lg px-4 py-3 text-slate-100 placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500" />
						{#if destSuggestions.length > 0}
							<div class="absolute z-10 w-full mt-1 bg-slate-800 border border-slate-700 rounded-lg shadow-lg overflow-hidden">
								{#each destSuggestions as s}
									<button type="button" on:click={() => selectDest(s)} class="w-full text-left px-4 py-2 hover:bg-slate-700 transition-colors">
										<span class="font-medium">{s.name}</span>
										<span class="text-slate-500 ml-2 text-sm">({s.id})</span>
									</button>
								{/each}
							</div>
						{/if}
					</div>
					<button type="submit" class="bg-blue-600 hover:bg-blue-700 text-white font-medium px-6 py-3 rounded-lg transition-colors whitespace-nowrap">Search</button>
				</div>
			</form>
		{:else}
			<div class="text-center py-8 text-slate-500">Loading stations...</div>
		{/if}

		{#if matches.length > 0}
			<div class="mb-8">
				<h2 class="text-lg font-semibold mb-4">Tables for {originQuery} → {destQuery}</h2>
				<div class="space-y-3">
					{#each matches as m}
						<a href="/table/{m.table}?from={fromCrs}&to={toCrs}" class="block bg-slate-800 border border-slate-700 rounded-lg p-4 hover:border-blue-500 transition-colors">
							<div class="flex items-center justify-between">
								<div>
									<span class="text-lg font-medium">Table {m.table}</span>
									{#if m.name}<span class="text-slate-400 ml-2">{m.name}</span>{/if}
								</div>
								<div class="flex items-center gap-3 text-sm text-slate-400">
									<span>{m.n_services} services</span>
									<span>{m.days.join(', ')}</span>
									{#if m.gap}<span class="text-amber-400">Gap</span>{/if}
								</div>
							</div>
							{#if m.operators.length > 0}
								<div class="mt-2 flex gap-2">{#each m.operators as op}
									<span class="text-xs bg-slate-700/50 px-2 py-1 rounded font-medium" style="color: {OP_COLORS[op] || DEFAULT_OP_COLOR}">{op}</span>
								{/each}</div>
							{/if}
						</a>
					{/each}
				</div>
			</div>
		{/if}

		{#if matches.length === 0 && fromCrs && toCrs}
			<div class="text-center py-8 text-slate-400">
				No matching tables found for this journey.
			</div>
		{/if}

		{#if !loaded || matches.length === 0}
			<div class="mb-8">
				<h2 class="text-lg font-semibold mb-4">Popular Routes</h2>
				<div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
					{#each popularRoutes as route}
						<button on:click={() => {
							fromCrs = route.from;
							toCrs = route.to;
							const fromStn = stations.find(s => s.id === route.from);
							const toStn = stations.find(s => s.id === route.to);
							originQuery = fromStn?.name ?? route.from;
							destQuery = toStn?.name ?? route.to;
							runSearch(fromCrs, toCrs);
						}}
							class="bg-slate-800 hover:bg-slate-700 border border-slate-700 rounded-lg px-4 py-3 text-left transition-colors">
							<span class="block font-medium">{route.label}</span>
						</button>
					{/each}
				</div>
				{#if gapCount > 0}
					<p class="mt-4 text-sm text-amber-400/70">{gapCount} timetable tables are not available in this dataset.</p>
				{/if}
			</div>
		{/if}

		<div class="text-center">
			<a href="/marey" class="text-blue-400 hover:text-blue-300 text-sm">Browse iBRY Marey Charts →</a>
		</div>
	</div>
</div>

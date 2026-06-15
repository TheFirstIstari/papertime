<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import Fuse from 'fuse.js';
	import { findMatchingTables, formatTime } from '$lib/search';
	import { loadStations, loadTableIndex, getGapCount } from '$lib/data';
	import type { StationEntry, TableEntry, TableMatch } from '$lib/types';

	let stations: StationEntry[] = [];
	let tableIndex: TableEntry[] = [];
	let fuse: Fuse<StationEntry> | null = null;

	let originQuery = '';
	let destQuery = '';
	let originSuggestions: StationEntry[] = [];
	let destSuggestions: StationEntry[] = [];
	let matches: TableMatch[] = [];
	let gapCount = 0;
	let loaded = false;

	onMount(async () => {
		[stations, tableIndex] = await Promise.all([loadStations(), loadTableIndex()]);
		fuse = new Fuse(stations, { keys: ['name', 'id'], threshold: 0.3 });
		gapCount = getGapCount(tableIndex);
		loaded = true;

		const params = $page.url.searchParams;
		const from = params.get('from');
		const to = params.get('to');
		if (from && to) {
			originQuery = from;
			destQuery = to;
			runSearch(from, to);
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

	function selectOrigin(s: StationEntry) {
		originQuery = s.name;
		originSuggestions = [];
	}

	function selectDest(s: StationEntry) {
		destQuery = s.name;
		destSuggestions = [];
	}

	function runSearch(from: string, to: string) {
		matches = findMatchingTables(from, to, stations, tableIndex);
	}

	function onSubmit() {
		const fromStation = stations.find((s) => s.name === originQuery || s.id === originQuery);
		const toStation = stations.find((s) => s.name === destQuery || s.id === destQuery);
		if (fromStation && toStation) {
			goto(`/?from=${fromStation.id}&to=${toStation.id}`);
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
		<!-- Header -->
		<div class="text-center mb-10">
			<h1 class="text-5xl font-bold tracking-tight">PaperTime</h1>
			<p class="text-xl text-slate-400 mt-2">May 2026 National Rail Timetables</p>
			<p class="text-sm text-slate-500 mt-1">Paper timetables · iBRY Marey graphs · Service pattern diagrams</p>
		</div>

		<!-- Search -->
		{#if loaded}
			<form on:submit|preventDefault={onSubmit} class="mb-8">
				<div class="flex flex-col sm:flex-row gap-3">
					<div class="relative flex-1">
						<input
							type="text"
							bind:value={originQuery}
							on:input={onOriginInput}
							placeholder="Origin station (e.g. London Euston)"
							class="w-full bg-slate-800 border border-slate-700 rounded-lg px-4 py-3 text-slate-100 placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
						/>
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
						<input
							type="text"
							bind:value={destQuery}
							on:input={onDestInput}
							placeholder="Destination station (e.g. Birmingham)"
							class="w-full bg-slate-800 border border-slate-700 rounded-lg px-4 py-3 text-slate-100 placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
						/>
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
					<button type="submit" class="bg-blue-600 hover:bg-blue-700 text-white font-medium px-6 py-3 rounded-lg transition-colors whitespace-nowrap">
						Search
					</button>
				</div>
			</form>
		{:else}
			<div class="text-center py-8 text-slate-500">Loading stations...</div>
		{/if}

		<!-- Results -->
		{#if matches.length > 0}
			<div class="mb-8">
				<h2 class="text-lg font-semibold mb-4">Tables for {originQuery} → {destQuery}</h2>
				<div class="space-y-3">
					{#each matches as m}
						<a href="/table/{m.table}?from={originQuery}&to={destQuery}" class="block bg-slate-800 border border-slate-700 rounded-lg p-4 hover:border-blue-500 transition-colors">
							<div class="flex items-center justify-between">
								<div>
									<span class="text-lg font-medium">Table {m.table}</span>
									{#if m.name}
										<span class="text-slate-400 ml-2">{m.name}</span>
									{/if}
								</div>
								<div class="flex items-center gap-3 text-sm text-slate-400">
									<span>{m.n_services} services</span>
									<span>{m.days.join(', ')}</span>
									{#if m.gap}
										<span class="text-amber-400">Gap</span>
									{/if}
								</div>
							</div>
							{#if m.operators.length > 0}
								<div class="mt-2 flex gap-2">
									{#each m.operators as op}
										<span class="text-xs bg-slate-700 px-2 py-1 rounded">{op}</span>
									{/each}
								</div>
							{/if}
						</a>
					{/each}
				</div>
			</div>
		{:else if originQuery && destQuery && originQuery !== destQuery}
			<div class="text-center py-8 text-slate-500">No matching tables found for this journey.</div>
		{/if}

		<!-- Popular routes -->
		{#if matches.length === 0}
			<div class="mb-8">
				<h2 class="text-lg font-semibold mb-4">Popular Routes</h2>
				<div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
					{#each popularRoutes as route}
						<button
							on:click={() => { originQuery = route.from; destQuery = route.to; runSearch(route.from, route.to); }}
							class="bg-slate-800 border border-slate-700 rounded-lg px-3 py-2 text-sm hover:border-blue-500 transition-colors text-left"
						>
							{route.label}
						</button>
					{/each}
				</div>
			</div>
		{/if}

		<!-- Gap notification -->
		{#if gapCount > 0}
			<div class="bg-amber-900/30 border border-amber-700/50 rounded-lg px-4 py-3 text-sm text-amber-200">
				{gapCount} timetable tables are not available in this dataset. These will be added when the full National Rail dataset is published.
			</div>
		{/if}
	</div>
</div>

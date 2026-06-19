<script lang="ts">
	export const csr = true;
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import Fuse from 'fuse.js';
	import { loadStationIndex } from '$lib/data';
	import type { Station } from '$lib/types';

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

	let stations: Station[] = [];
	let fuse: Fuse<Station> | null = null;
	let query = $state('');
	let suggestions: Station[] = $state([]);
	let loaded = $state(false);

	onMount(async () => {
		try {
			stations = await loadStationIndex();
			fuse = new Fuse(stations, { keys: ['name', 'crs', 'id'], threshold: 0.3 });
			loaded = true;
		} catch (err) {
			console.error('PaperTime init failed:', err);
		}
	});

	function onInput() {
		if (!fuse || query.length < 2) {
			suggestions = [];
			return;
		}
		suggestions = fuse.search(query).map((r) => r.item).slice(0, 8);
	}

	function selectStation(s: Station) {
		query = s.name;
		suggestions = [];
		goto(`/station/${s.crs}`);
	}
</script>

<svelte:head>
	<title>PaperTime — National Rail Timetables</title>
	<meta name="description" content="Explore National Rail timetables — station views, Marey charts, and service patterns." />
</svelte:head>

<div class="min-h-screen bg-slate-900 text-slate-100">
	<div class="max-w-4xl mx-auto px-4 py-12">
		<div class="text-center mb-10">
			<h1 class="text-5xl font-bold tracking-tight">PaperTime</h1>
			<p class="text-xl text-slate-400 mt-2">National Rail Timetables</p>
			<p class="text-sm text-slate-500 mt-1">Station views · Marey charts · Service patterns</p>
		</div>

		{#if loaded}
			<div class="relative max-w-xl mx-auto mb-8">
				<input
					type="text"
					bind:value={query}
					on:input={onInput}
					placeholder="Search for a station (e.g. London Euston)"
					class="w-full bg-slate-800 border border-slate-700 rounded-lg px-4 py-3 text-slate-100 placeholder-slate-500 focus:outline-none focus:ring-2 focus:ring-blue-500"
				/>
				{#if suggestions.length > 0}
					<div class="absolute z-10 w-full mt-1 bg-slate-800 border border-slate-700 rounded-lg shadow-lg overflow-hidden">
						{#each suggestions as s}
							<button
								type="button"
								on:click={() => selectStation(s)}
								class="w-full text-left px-4 py-2 hover:bg-slate-700 transition-colors"
							>
								<span class="font-medium">{s.name}</span>
								<span class="text-slate-500 ml-2 text-sm">({s.crs})</span>
								{#if s.n_services > 0}
									<span class="text-slate-600 ml-2 text-xs">{s.n_services} services</span>
								{/if}
							</button>
						{/each}
					</div>
				{/if}
			</div>
		{:else}
			<div class="text-center py-8 text-slate-500">Loading stations...</div>
		{/if}

		{#if loaded}
			<div class="text-center text-sm text-slate-500">
				{stations.length} stations available
			</div>
		{/if}
	</div>
</div>

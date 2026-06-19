<script lang="ts">
	export const csr = true;
	import { onMount } from 'svelte';
	import { loadStationServices } from '$lib/data';
	import type { StationServices, ServiceRef, CallRef } from '$lib/types';

	let { data } = $props<{ data: { crs: string } }>();
	let crs = data.crs;
	let stationData: StationServices | null = null;
	let loading = $state(true);
	let error = $state('');
	let filterOperator = $state('');
	let filterDest = $state('');

	const OP_COLORS: Record<string, string> = {
		'CC': '#009E73', 'XC': '#009E73', 'SE': '#009E73', 'LE': '#009E73',
		'EM': '#CC79A7', 'GR': '#CC79A7', 'AW': '#CC79A7',
		'LO': '#E86A10', 'ME': '#E86A10',
		'VT': '#E32636', 'HX': '#E32636', 'HT': '#E32636',
		'GW': '#56B4E9', 'SR': '#56B4E9',
		'TP': '#D55E00', 'TL': '#D55E00', 'LM': '#D55E00',
		'NT': '#0072B2', 'SW': '#0072B2', 'CH': '#0072B2',
		'SN': '#F0E442', 'GN': '#F0E442',
	};

	function formatTime(t: string | null): string {
		if (!t) return '---';
		return t;
	}

	function getDepTime(s: ServiceRef): string {
		const call = s.calls.find(c => c.crs === crs);
		return call?.dep ? formatTime(call.dep) : '---';
	}

	function getArrTime(s: ServiceRef): string {
		const call = s.calls.find(c => c.crs === crs);
		return call?.arr ? formatTime(call.arr) : '---';
	}

	let filteredServices = $derived(
		stationData?.services?.filter(s => {
			if (filterOperator && s.operator !== filterOperator) return false;
			if (filterDest && s.destination !== filterDest) return false;
			return true;
		}) ?? []
	);

	let operators = $derived([...new Set(stationData?.services?.map(s => s.operator) ?? [])].sort());
	let destinations = $derived([...new Set(stationData?.services?.map(s => s.destination_name) ?? [])].sort());

	onMount(async () => {
		try {
			stationData = await loadStationServices(crs);
		} catch (err) {
			error = err instanceof Error ? err.message : 'Failed to load station data';
		}
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
			<h1 class="text-3xl font-bold mb-2">{stationData.name}</h1>
			<p class="text-slate-400 mb-6">{stationData.services.length} services</p>

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
						{#each filteredServices as svc (svc.id)}
							<tr class="border-b border-slate-800 hover:bg-slate-800/50">
								<td class="py-2 pr-4 font-mono">{getDepTime(svc)}</td>
								<td class="py-2 pr-4">{svc.headcode}</td>
								<td class="py-2 pr-4">
									<span class="px-1.5 py-0.5 rounded text-xs font-medium" style="color: {OP_COLORS[svc.operator] || '#64748b'}">
										{svc.operator}
									</span>
								</td>
								<td class="py-2 pr-4">{svc.destination_name}</td>
								<td class="py-2 pr-4 text-slate-400 text-xs">
									{svc.calls.slice(1, -1).map(c => c.crs).join(', ')}
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>

			{#if filteredServices.length === 0}
				<div class="text-center py-8 text-slate-400">No services match your filters.</div>
			{/if}
		{/if}
	</div>
</div>

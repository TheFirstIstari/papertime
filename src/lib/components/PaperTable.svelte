<script lang="ts">
	import { onMount } from 'svelte';
	import { loadTable } from '$lib/data';
	import { formatTime } from '$lib/search';
	import type { TableData } from '$lib/types';

	let { tableNum, tableData: initialData, fromCrs, toCrs } = $props<{
		tableNum?: string;
		tableData?: TableData | null;
		fromCrs?: string;
		toCrs?: string;
	}>();

	let table = $state<TableData | null>(initialData ?? null);
	let loading = $state(!initialData);
	let error = $state('');
	let activeDay = $state('MF');
	let highlightedCol = $state<number | null>(null);
	let highlightedRow = $state<string | null>(null);
	let sortBy = $state<'dep' | 'arr' | 'none'>('none');
	let searchQuery = $state('');

	onMount(async () => {
		if (initialData) {
			if (initialData.days?.length) activeDay = initialData.days[0];
			loading = false;
			return;
		}
		if (!tableNum) { error = 'No table specified'; loading = false; return; }
		try {
			table = await loadTable(tableNum);
			if (table?.days?.length) activeDay = table.days[0];
		} catch (e) {
			error = 'Failed to load timetable data';
		}
		loading = false;
	});

	function getServicesForDay(day: string) {
		if (!table) return [];
		return table.services.filter((s) => s.days.includes(day));
	}

	function getStationRows() {
		if (!table) return [];
		const seen = new Set<string>();
		const rows: { crs: string; name: string; isFrom: boolean; isTo: boolean }[] = [];
		for (const crs of table.stations) {
			if (seen.has(crs)) continue;
			seen.add(crs);
			rows.push({ crs, name: crs, isFrom: crs === fromCrs, isTo: crs === toCrs });
		}
		return rows;
	}

	function getFilteredServices() {
		if (!table) return [];
		let svcs = table.services.filter((s) => s.days.includes(activeDay));
		if (searchQuery) {
			const q = searchQuery.toLowerCase();
			svcs = svcs.filter((s) =>
				s.id.toLowerCase().includes(q) ||
				s.operator.toLowerCase().includes(q) ||
				s.stops.some((st) => st.station.toLowerCase().includes(q))
			);
		}
		if (sortBy === 'dep') {
			svcs = [...svcs].sort((a, b) => {
				const aMin = a.stops.find((s) => s.dep !== null)?.dep ?? 9999;
				const bMin = b.stops.find((s) => s.dep !== null)?.dep ?? 9999;
				return aMin - bMin;
			});
		} else if (sortBy === 'arr') {
			svcs = [...svcs].sort((a, b) => {
				const aArr = [...a.stops].reverse().find((s) => s.arr !== null)?.arr ?? 9999;
				const bArr = [...b.stops].reverse().find((s) => s.arr !== null)?.arr ?? 9999;
				return aArr - bArr;
			});
		}
		return svcs;
	}

	let services = $derived(getFilteredServices());
	let stationRows = $derived(getStationRows());
</script>

{#if loading}
	<div class="text-center py-12 text-slate-400">Loading timetable...</div>
{:else if error}
	<div class="text-center py-12 text-red-400">{error}</div>
{:else if table}
	<div class="max-w-6xl mx-auto px-4 py-8">
		<div class="mb-6">
			<a href="/" class="text-blue-400 hover:text-blue-300 text-sm mb-2 inline-block">&larr; Back to search</a>
			<h1 class="text-3xl font-bold">Table {table.table}</h1>
			{#if table.name}<p class="text-slate-400 mt-1">{table.name}</p>{/if}
			<div class="flex gap-2 mt-3">
				{#each table.operators as op}
					<span class="text-xs bg-slate-700 px-2 py-1 rounded" style="color: {op.color}">{op.name || op.code}</span>
				{/each}
			</div>
		</div>

		<div class="flex flex-wrap gap-3 mb-4">
			<div class="flex gap-1">
				{#each table.days as day}
					<button on:click={() => activeDay = day}
						class="px-3 py-1.5 rounded text-sm font-medium transition-colors {activeDay === day ? 'bg-blue-600 text-white' : 'bg-slate-800 text-slate-300 hover:bg-slate-700'}"
					>{day}</button>
				{/each}
			</div>
			<select bind:value={sortBy} class="bg-slate-800 border border-slate-700 rounded px-3 py-1.5 text-sm text-slate-300">
				<option value="none">Default order</option>
				<option value="dep">Sort by departure</option>
				<option value="arr">Sort by arrival</option>
			</select>
			<input type="text" bind:value={searchQuery} placeholder="Search services..."
				class="bg-slate-800 border border-slate-700 rounded px-3 py-1.5 text-sm text-slate-100 placeholder-slate-500 focus:outline-none focus:ring-1 focus:ring-blue-500" />
		</div>

		<div class="overflow-x-auto">
			<table class="w-full border-collapse text-sm">
				<thead><tr>
					<th class="sticky left-0 bg-slate-900 border-b border-r border-slate-700 px-3 py-2 text-left font-medium text-slate-400 z-10 no-print">Station</th>
					{#each services as svc, colIdx}
						<th on:click={() => highlightedCol = highlightedCol === colIdx ? null : colIdx}
							class="border-b border-slate-700 px-2 py-2 text-center font-mono text-xs cursor-pointer transition-colors {highlightedCol === colIdx ? 'bg-blue-900/50' : 'hover:bg-slate-800'}">
							<span class="no-print">{svc.operator || '—'}</span>
							<span class="print-only">{svc.id}</span>
						</th>
					{/each}
				</tr></thead>
				<tbody>
					{#each stationRows as row}
						<tr on:click={() => highlightedRow = highlightedRow === row.crs ? null : row.crs}
							class="cursor-pointer transition-colors {highlightedRow === row.crs ? 'bg-slate-800' : 'hover:bg-slate-800/50'} {row.isFrom ? 'bg-green-900/20' : ''} {row.isTo ? 'bg-red-900/20' : ''}">
							<td class="sticky left-0 bg-slate-900 border-r border-slate-700 px-3 py-2 font-medium z-10">
								{row.name}
								{#if row.isFrom}<span class="text-green-400 text-xs ml-1">▲</span>{/if}
								{#if row.isTo}<span class="text-red-400 text-xs ml-1">▼</span>{/if}
							</td>
							{#each services as svc, colIdx}
								{@const stop = svc.stops.find((s) => s.station === row.crs)}
								<td class="border-b border-slate-800 px-2 py-2 text-center font-mono text-xs {highlightedCol === colIdx ? 'bg-blue-900/30' : ''}">
									{#if stop}
										{#if stop.dep !== null}<span class="text-slate-200">{formatTime(stop.dep)}</span>
										{:else if stop.arr !== null}<span class="text-slate-400">{formatTime(stop.arr)}</span>
										{:else}<span class="text-slate-600">—</span>{/if}
									{:else}<span class="text-slate-700">—</span>{/if}
								</td>
							{/each}
						</tr>
					{/each}
				</tbody>
			</table>
		</div>

		<div class="mt-4 flex flex-wrap gap-4 text-xs text-slate-500 no-print">
			<span><span class="text-green-400">▲</span> Origin</span>
			<span><span class="text-red-400">▼</span> Destination</span>
			<span>Click column header to highlight service</span>
			<span>Click row to highlight station</span>
		</div>
	</div>
{/if}

<style>
	@media print {
		.no-print { display: none !important; }
		.print-only { display: inline !important; }
		body { background: white !important; color: black !important; }
		table { font-size: 9px; width: 100%; }
		th, td { border: 1px solid #ccc !important; padding: 2px 4px !important; }
		.sticky { position: static !important; }
		tr { page-break-inside: avoid; }
		@page { size: A4 landscape; margin: 10mm; }
	}
	.print-only { display: none; }
</style>

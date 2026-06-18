<script lang="ts">
	import { onMount } from 'svelte';

	interface MareyEntry {
		table: string;
		route: string;
		route_id: string;
		stations: { crs: string; mileage: number }[];
		services: { days: string[] }[];
	}

	let charts = $state<MareyEntry[]>([]);
	let loading = $state(true);

	onMount(async () => {
		try {
			const resp = await fetch('/data/marey-index.json');
			if (resp.ok) {
				charts = await resp.json();
			} else {
				// Build index from individual files
				const indexResp = await fetch('/data/route-index.json');
				if (indexResp.ok) {
					const routeIndex = await indexResp.json();
					charts = routeIndex
						.filter((r: any) => r.file && !r.gap)
						.map((r: any) => ({
							table: r.table,
							route: r.name || `Route ${r.table}`,
							route_id: r.table,
							n_services: r.n_services,
							days: r.days
						}));
				}
			}
		} catch (e) {
			console.error('Failed to load Marey index');
		}
		loading = false;
	});
</script>

<svelte:head>
	<title>Marey Charts — PaperTime</title>
</svelte:head>

<div class="min-h-screen bg-slate-900 text-slate-100">
	<div class="max-w-4xl mx-auto px-4 py-12">
		<a href="/" class="text-blue-400 hover:text-blue-300 text-sm mb-4 inline-block">&larr; Back to search</a>
		<h1 class="text-3xl font-bold mb-2">iBRY Marey Charts</h1>
		<p class="text-slate-400 mb-6">Time–distance diagrams showing train services as slanted lines. Steeper = faster.</p>

		{#if loading}
			<div class="text-center py-8 text-slate-400">Loading...</div>
		{:else if charts.length === 0}
			<div class="text-center py-8 text-slate-400">No Marey charts available yet.</div>
		{:else}
			<div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
				{#each charts as chart}
					<a href="/marey/t{chart.table}"
						class="bg-slate-800 border border-slate-700 rounded-lg p-4 hover:border-blue-500 transition-colors">
						<div class="font-medium">Table {chart.table}</div>
						<div class="text-sm text-slate-400 mt-1">{chart.route}</div>
						<div class="text-xs text-slate-500 mt-2">
							{#if chart.n_services}{chart.n_services} services{/if}
							{#if chart.days?.length} · {chart.days.join(', ')}{/if}
						</div>
					</a>
				{/each}
			</div>
		{/if}
	</div>
</div>

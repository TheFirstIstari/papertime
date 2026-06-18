<script lang="ts">
	import { onMount } from 'svelte';
	import PaperTable from '$lib/components/PaperTable.svelte';
	import { loadTable } from '$lib/data';

	let { id, from: fromCrs, to: toCrs } = $props<{ id: string; from: string; to: string }>();

	onMount(async () => {
		try {
			table = await loadTable(id);
		} catch (e) {
			error = 'Timetable data not available for this table';
		}
		loading = false;
	});
</script>

<svelte:head>
	<title>Table {id} — PaperTime</title>
</svelte:head>

<div class="min-h-screen bg-slate-900 text-slate-100">
	<div class="max-w-6xl mx-auto px-4 py-8">
		<a href="/" class="text-blue-400 hover:text-blue-300 text-sm mb-4 inline-block">&larr; Back to search</a>

		{#if loading}
			<div class="text-center py-12 text-slate-400">Loading timetable...</div>
		{:else if error}
			<h1 class="text-3xl font-bold mb-2">Table {id}</h1>
			<p class="text-slate-400 mb-4">{error}</p>
			<div class="bg-amber-900/30 border border-amber-700/50 rounded-lg px-4 py-3 text-sm text-amber-200 mb-6">
				This timetable is not available in the current dataset.
			</div>
			<a href="/" class="bg-blue-600 hover:bg-blue-700 text-white font-medium px-4 py-2 rounded-lg transition-colors inline-block">
				Search for another journey
			</a>
		{:else if table}
			<PaperTable tableData={table} {fromCrs} {toCrs} />

			<div class="mt-6 no-print">
				<a href="/marey/t{id}" class="text-blue-400 hover:text-blue-300 text-sm">
					View as iBRY Marey chart &rarr;
				</a>
			</div>
		{/if}
	</div>
</div>

<script lang="ts">
	export const prerender = false;

	import { onMount } from 'svelte';
	import MareyChart from '$lib/components/MareyChart.svelte';

	let { id } = $props<{ id: string }>();
	let mareyData = $state<any>(null);
	let loading = $state(true);
	let error = $state('');

	onMount(async () => {
		try {
			const resp = await fetch(`/marey/${id}.json`);
			if (!resp.ok) throw new Error('Not found');
			mareyData = await resp.json();
		} catch (e) {
			error = 'Marey chart data not available for this route';
		}
		loading = false;
	});
</script>

<svelte:head>
	<title>Marey Chart {id} — PaperTime</title>
</svelte:head>

<div class="min-h-screen bg-slate-900 text-slate-100">
	<div class="max-w-6xl mx-auto px-4 py-8">
		<a href="/" class="text-blue-400 hover:text-blue-300 text-sm mb-4 inline-block">&larr; Back to search</a>
		<h1 class="text-3xl font-bold mb-2">Marey Chart — {id}</h1>
		<p class="text-slate-400 mb-6">iBRY time–distance diagram</p>

		{#if loading}
			<div class="text-center py-12 text-slate-400">Loading chart...</div>
		{:else if error}
			<div class="text-center py-12 text-red-400">{error}</div>
		{:else}
			<MareyChart {mareyData} />
		{/if}
	</div>
</div>

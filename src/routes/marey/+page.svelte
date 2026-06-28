<script lang="ts">
	import { onMount } from 'svelte';

	interface StationEntry {
		id: string;
		name: string;
		n_services: number;
		type: string;
	}

	let stations = $state<StationEntry[]>([]);
	let loading = $state(true);
	let search = $state('');
	let sortBy = $state<'name' | 'services'>('services');

	let filtered = $derived(
		stations
			.filter((s) => !search || s.name.toLowerCase().includes(search.toLowerCase()) || s.id.toLowerCase().includes(search.toLowerCase()))
			.sort((a, b) => (sortBy === 'services' ? b.n_services - a.n_services : a.name.localeCompare(b.name)))
	);

	onMount(async () => {
		try {
			const resp = await fetch('/station-index.json');
			if (resp.ok) {
				const data = await resp.json();
				stations = data
					.filter((s: any) => s.n_services > 0)
					.map((s: any) => ({
						id: s.id,
						name: s.name,
						n_services: s.n_services,
						type: s.type
					}));
			}
		} catch (e) {
			console.error('Failed to load station index');
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
		<p class="text-slate-400 mb-6">Time–distance diagrams showing train services as slanted lines.</p>

		<div class="flex gap-3 mb-6">
			<input
				type="text"
				placeholder="Search stations..."
				bind:value={search}
				class="flex-1 bg-slate-800 border border-slate-700 rounded-lg px-4 py-2 text-sm focus:border-blue-500 outline-none"
			/>
			<select bind:value={sortBy} class="bg-slate-800 border border-slate-700 rounded-lg px-3 py-2 text-sm">
				<option value="services">Sort by services</option>
				<option value="name">Sort by name</option>
			</select>
		</div>

		{#if loading}
			<div class="text-center py-8 text-slate-400">Loading...</div>
		{:else if filtered.length === 0}
			<div class="text-center py-8 text-slate-400">No Marey charts available.</div>
		{:else}
			<div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
				{#each filtered as station}
					<a href="/marey/{station.id}"
						class="bg-slate-800 border border-slate-700 rounded-lg p-4 hover:border-blue-500 transition-colors">
						<div class="font-medium">{station.name}</div>
						<div class="text-sm text-slate-400 mt-1">{station.id} · {station.type}</div>
						<div class="text-xs text-slate-500 mt-2">{station.n_services} services</div>
					</a>
				{/each}
			</div>
		{/if}
	</div>
</div>

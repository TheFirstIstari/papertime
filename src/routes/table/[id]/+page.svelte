<script lang="ts">
	let { id } = $props<{ id: string }>();
</script>

<svelte:head>
	<title>Table {id} — PaperTime</title>
</svelte:head>

<div class="min-h-screen bg-slate-900 text-slate-100">
	<div class="max-w-4xl mx-auto px-4 py-12">
		<a href="/" class="text-blue-400 hover:text-blue-300 text-sm mb-4 inline-block">&larr; Back to search</a>
		<h1 class="text-3xl font-bold mb-2">Table {id}</h1>
		<p class="text-slate-400 mb-4">This timetable is not available in the current dataset.</p>
		<div class="bg-amber-900/30 border border-amber-700/50 rounded-lg px-4 py-3 text-sm text-amber-200 mb-6">
			One of the 29 missing timetable tables. These will be added when the full National Rail dataset is published.
		</div>
		<a href="/" class="bg-blue-600 hover:bg-blue-700 text-white font-medium px-4 py-2 rounded-lg transition-colors inline-block">
			Search for another journey
		</a>
	</div>
</div>

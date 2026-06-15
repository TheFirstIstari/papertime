<script lang="ts">
	import PaperTable from '$lib/components/PaperTable.svelte';
	import { page } from '$app/stores';

	let { id } = $props<{ id: string }>();

	const fromCrs = $page.url.searchParams.get('from') || undefined;
	const toCrs = $page.url.searchParams.get('to') || undefined;
</script>

<PaperTable tableNum={id} {fromCrs} {toCrs} />

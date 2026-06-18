import { findMatchingTables, formatTime } from '$lib/search';
import { loadStations, loadTableIndex, getGapCount } from '$lib/data';
import type { StationEntry, TableEntry, TableMatch } from '$lib/types';
import Fuse from 'fuse.js';

let stations: StationEntry[] = [];
let tableIndex: TableEntry[] = [];
let fuse: Fuse<StationEntry> | null = null;
let originQuery = $state('');
let destQuery = $state('');
let fromCrs = $state('');
let toCrs = $state('');
let originSuggestions: StationEntry[] = $state([]);
let destSuggestions: StationEntry[] = $state([]);
let matches: TableMatch[] = $state([]);
let gapCount = $state(0);
let loaded = $state(false);

onMount(async () => {
	try {
		[stations, tableIndex] = await Promise.all([loadStations(), loadTableIndex()]);
		fuse = new Fuse(stations, { keys: ['name', 'id'], threshold: 0.3 });
		gapCount = getGapCount(tableIndex);
		loaded = true;
		const params = $page.url.searchParams;
		const from = params.get('from');
		const to = params.get('to');
		if (from && to) {
			fromCrs = from;
			toCrs = to;
			const fromStn = stations.find(s => s.id === from);
			const toStn = stations.find(s => s.id === to);
			originQuery = fromStn?.name ?? from;
			destQuery = toStn?.name ?? to;
			runSearch(fromCrs, toCrs);
		}
	} catch (err) {
		console.error('Page init failed:', err);
	}
});

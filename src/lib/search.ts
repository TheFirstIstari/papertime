import type { StationEntry, TableEntry, TableMatch } from './types';

export function findMatchingTables(fromCrs: string, toCrs: string, stations: StationEntry[], tableIndex: TableEntry[]): TableMatch[] {
  const fromStation = stations.find(s => s.id === fromCrs);
  const toStation = stations.find(s => s.id === toCrs);

  if (!fromStation || !toStation) {
    return [];
  }

  const fromTables = new Set(fromStation.tables);
  const toTables = new Set(toStation.tables);
  const matchingTables = [...fromTables].filter(table => toTables.has(table));

  const matches: TableMatch[] = matchingTables.map(tableId => {
    const tableEntry = tableIndex.find(t => t.table === tableId);
    if (!tableEntry) {
      return null; // will be filtered out
    }
    return {
      table: tableEntry.table,
      name: tableEntry.name,
      operators: tableEntry.operators,
      n_services: tableEntry.n_services,
      days: tableEntry.days,
      hasRouteMap: tableEntry.has_route_map,
      gap: tableEntry.gap,
      stationCount: tableEntry.stations.length
    };
  }).filter((match): match is TableMatch => match !== null);

  // Sort by: !gap first (so gap=false comes first), then n_services descending
  matches.sort((a, b) => {
    if (a.gap !== b.gap) {
      return a.gap ? 1 : -1; // false (non-gap) comes first
    }
    return b.n_services - a.n_services;
  });

  return matches;
}

export function formatTime(mins: number | null): string {
  if (mins === null) {
    return '---';
  }
  const hours = Math.floor(mins / 60);
  const minutes = mins % 60;
  return `${hours.toString().padStart(2, '0')}:${minutes.toString().padStart(2, '0')}`;
}
import type { Station, StationEntry, TableEntry, TableMatch } from './types';

// Station-centric search

export function findStationsByQuery(query: string, stations: Station[]): Station[] {
  if (!query || query.length < 2) return [];
  const q = query.toLowerCase();
  return stations
    .filter(s => 
      s.name.toLowerCase().includes(q) || 
      s.tiploc.toLowerCase().includes(q) ||
      s.id.toLowerCase().includes(q)
    )
    .slice(0, 10);
}

export function getStationByCrs(crs: string, stations: Station[]): Station | undefined {
  return stations.find(s => s.tiploc === crs || s.id === crs);
}

// Legacy table-centric search (kept for compatibility)

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
      return null;
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

  matches.sort((a, b) => {
    if (a.gap !== b.gap) {
      return a.gap ? 1 : -1;
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

import type { Station, StationIndex, StationServices, TableEntry, TableData } from './types';

// Station-centric data loading

export async function loadStationIndex(): Promise<Station[]> {
  const response = await fetch('/station-index.json');
  if (!response.ok) {
    throw new Error(`Failed to load station index: ${response.status}`);
  }
  const data: StationIndex = await response.json();
  return data.stations;
}

export async function loadStationServices(crs: string): Promise<StationServices> {
  const response = await fetch(`/services/${crs}.json`);
  if (!response.ok) {
    throw new Error(`Failed to load services for ${crs}: ${response.status}`);
  }
  return response.json();
}

// Legacy functions (kept for compatibility during migration)

export async function loadStations(): Promise<import('./types').StationEntry[]> {
  const response = await fetch('/stations.json');
  if (!response.ok) {
    throw new Error(`Failed to load stations: ${response.status}`);
  }
  return response.json();
}

export async function loadTableIndex(): Promise<TableEntry[]> {
  const response = await fetch('/table-index.json');
  if (!response.ok) {
    throw new Error(`Failed to load table index: ${response.status}`);
  }
  return response.json();
}

export async function loadTable(tableNum: string): Promise<TableData> {
  const response = await fetch(`/services/${tableNum}.json`);
  if (!response.ok) {
    throw new Error(`Failed to load table ${tableNum}: ${response.status}`);
  }
  return response.json();
}

export function getGapCount(tableIndex: TableEntry[]): number {
  return tableIndex.filter(entry => entry.gap).length;
}

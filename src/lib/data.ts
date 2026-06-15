import type { StationEntry, TableEntry, TableData } from './types';

export async function loadStations(): Promise<StationEntry[]> {
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
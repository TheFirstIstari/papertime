// Station-centric data types for PaperTime

export interface Station {
  id: string;
  name: string;
  tiploc: string;
  lat: number | null;
  lng: number | null;
  type: string;
  n_services: number;
  operators: string[];
  destinations: string[];
  file: string;
}

export interface StationIndex extends Array<Station> {}

export interface ServiceRef {
  id: string;
  headcode: string;
  operator: string;
  origin: string;
  origin_name: string;
  destination: string;
  destination_name: string;
  calls: CallRef[];
  days: string[];
}

export interface CallRef {
  crs: string;
  arr: string | null;   // HH:MM format
  dep: string | null;   // HH:MM format
}

export interface StationServices {
  station: string;
  name: string;
  services: ServiceRef[];
}

// Legacy types (kept for compatibility during migration)
export interface StationEntry {
  id: string;
  name: string;
  aliases: string[];
  tables: string[];
  routes: string[];
  lat: number | null;
  lng: number | null;
  type: string;
}

export interface TableEntry {
  table: string;
  name: string | null;
  region: string | null;
  operators: string[];
  stations: string[];
  n_services: number;
  days: string[];
  file: string | null;
  routes: string[];
  has_route_map: boolean;
  gap: boolean;
}

export interface OperatorInfo {
  code: string;
  name: string;
  color: string;
}

export interface ServiceStop {
  station: string;
  arr: number | null;
  dep: number | null;
}

export interface Service {
  id: string;
  headcode: string;
  operator: string;
  days: string[];
  direction: string;
  stops: ServiceStop[];
}

export interface TableData {
  table: string;
  name: string;
  period: string;
  operators: OperatorInfo[];
  days: string[];
  stations: string[];
  services: Service[];
  gap: boolean;
}

export interface TableMatch {
  table: string;
  name: string | null;
  operators: string[];
  n_services: number;
  days: string[];
  hasRouteMap: boolean;
  gap: boolean;
  stationCount: number;
}

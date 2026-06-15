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
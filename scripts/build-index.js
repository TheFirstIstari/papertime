#!/usr/bin/env node
// scripts/build-index.js
// Generate station-index.json from individual service files

import { readdirSync, readFileSync, writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

const servicesDir = join(process.cwd(), 'static', 'services');
const outputPath = join(process.cwd(), 'static', 'station-index.json');

const stations = new Map();

// Read all service files
const files = readdirSync(servicesDir).filter(f => f.endsWith('.json'));
console.log(`Processing ${files.length} service files...`);

for (const file of files) {
  try {
    const data = JSON.parse(readFileSync(join(servicesDir, file), 'utf-8'));
    const crs = file.replace('.json', '');
    
    if (!stations.has(crs)) {
      stations.set(crs, {
        id: crs,
        name: data.station || crs,
        tiploc: data.tiploc || crs,
        lat: null,
        lng: null,
        station_type: 'minor',
        n_services: 0,
        operators: new Set(),
        destinations: new Set(),
      });
    }
    
    const station = stations.get(crs);
    station.n_services = data.services?.length || 0;
    
    if (data.services) {
      for (const svc of data.services) {
        if (svc.operator) station.operators.add(svc.operator);
        if (svc.destination_name) station.destinations.add(svc.destination_name);
        else if (svc.destination) station.destinations.add(svc.destination);
      }
    }
  } catch (e) {
    console.warn(`Failed to process ${file}: ${e.message}`);
  }
}

// Convert to array and finalize
const index = Array.from(stations.values())
  .map(s => ({
    ...s,
    operators: Array.from(s.operators).sort(),
    destinations: Array.from(s.destinations).sort(),
    file: `services/${s.id}.json`,
  }))
  .sort((a, b) => a.id.localeCompare(b.id));

writeFileSync(outputPath, JSON.stringify(index));
console.log(`Wrote ${index.length} stations to station-index.json`);

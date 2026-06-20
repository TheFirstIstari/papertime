#!/usr/bin/env node
// scripts/build-index.js
// Generate station-index.json from individual service files
// Only extracts metadata, not full service details

import { readdirSync, readFileSync, writeFileSync } from 'fs';
import { join } from 'path';

const servicesDir = join(process.cwd(), 'static', 'services');
const outputPath = join(process.cwd(), 'static', 'station-index.json');

const stations = new Map();

let files;
try {
  files = readdirSync(servicesDir).filter(f => f.endsWith('.json'));
} catch (e) {
  console.warn('No services directory found, creating empty index');
  writeFileSync(outputPath, '[]');
  process.exit(0);
}

console.log(`Processing ${files.length} service files...`);

for (const file of files) {
  try {
    const data = JSON.parse(readFileSync(join(servicesDir, file), 'utf-8'));
    const id = file.replace('.json', '');
    
    // Handle both old format (table, name) and new format (station, name)
    const name = data.station || data.name || id;
    const services = data.services || [];
    
    if (!stations.has(id)) {
      stations.set(id, {
        id,
        name,
        tiploc: data.tiploc || id,
        lat: null,
        lng: null,
        type: 'minor',
        n_services: 0,
        operators: [],
        destinations: [],
        file: `services/${id}.json`,
      });
    }
    
    const station = stations.get(id);
    station.n_services = services.length;
    
    // Extract operators and destinations
    const ops = new Set(station.operators);
    const dests = new Set(station.destinations);
    
    for (const svc of services) {
      if (svc.operator) ops.add(svc.operator);
      if (svc.destination_name) dests.add(svc.destination_name);
      else if (svc.destination) dests.add(svc.destination);
    }
    
    station.operators = Array.from(ops).sort();
    station.destinations = Array.from(dests).sort();
  } catch (e) {
    console.warn(`Failed to process ${file}: ${e.message}`);
  }
}

const index = Array.from(stations.values())
  .sort((a, b) => a.id.localeCompare(b.id));

writeFileSync(outputPath, JSON.stringify(index));
console.log(`Wrote ${index.length} stations to station-index.json`);

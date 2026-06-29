#!/usr/bin/env node
/**
 * Build CRS → TIPLOC mapping from service data.
 * Each station file static/services/{crs}.json contains services passing through that station.
 * The calls use TIPLOC codes. Scan each file to find what TIPLOC code represents
 * the station itself (it's the one not shared with other files' own identity).
 *
 * ponytail: sample first 100 services per file, find TIPLOC that appears in 100% of calls.
 */

import { readdir, readFile } from 'fs/promises';
import { join } from 'path';

const STATIC_DIR = join(process.cwd(), 'static');
const SERVICES_DIR = join(STATIC_DIR, 'services');
const STATION_INDEX_PATH = join(STATIC_DIR, 'station-index.json');

async function build() {
  const stationIndex = JSON.parse(await readFile(STATION_INDEX_PATH, 'utf-8'));
  const crsList = stationIndex.map(s => s.id);

  const files = (await readdir(SERVICES_DIR)).filter(f => f.endsWith('.json'));
  const mapping = {};

  for (const file of files) {
    const crs = file.replace('.json', '');
    const data = JSON.parse(await readFile(join(SERVICES_DIR, file), 'utf-8'));
    const services = (data.services || []).slice(0, 100);

    if (services.length === 0) continue;

    // For each service, find the call that matches this station by checking if
    // the call's CRS is NOT any other station's file ID (it's a TIPLOC)
    // ponytairl: just collect TIPLOC that appears in every service
    const firstCalls = services[0].calls?.map(c => c.crs) || [];
    for (const candidate of firstCalls) {
      if (services.every(s => s.calls?.some(c => c.crs === candidate))) {
        // This CRS/TIPLOC appears in EVERY service → it IS this station
        if (candidate !== crs && candidate.length > 3) {
          // It's a TIPLOC (different from file ID CRS and longer than 3 chars)
          mapping[crs] = candidate;
        } else if (!mapping[crs]) {
          mapping[crs] = candidate; // same ID, keep as-is
        }
      }
    }
  }

  console.log(`Mapped ${Object.keys(mapping).length} CRS→TIPLOC`);
  // Show some
  for (const [k, v] of Object.entries(mapping).slice(0, 10)) {
    console.log(`  ${k} → ${v}`);
  }

  return mapping;
}

build();

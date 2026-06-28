#!/usr/bin/env node
/**
 * build-index.js — Generate station-index.json from service files
 * 
 * Reads all static/services/*.json, aggregates station metadata,
 * enriches with NaPTAN coordinates, and writes static/station-index.json
 * 
 * Usage: node scripts/build-index.js
 */

import { readdir, readFile, writeFile, mkdir } from 'fs/promises';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const STATIC_DIR = join(__dirname, '..', 'static');
const SERVICES_DIR = join(STATIC_DIR, 'services');
const OUTPUT_PATH = join(STATIC_DIR, 'station-index.json');

// NaPTAN TIPLOC → {name, lat, lng} mapping
// Generated from naptan package: ATCO codes starting with "9100" + TIPLOC
// This is a curated subset — full NaPTAN data has ~2,700 rail stations
const NAptanData = await loadNaptanData();

async function loadNaptanData() {
  // Try to load from a local cache file first
  try {
    const cachePath = join(STATIC_DIR, 'naptan-cache.json');
    const cache = JSON.parse(await readFile(cachePath, 'utf-8'));
    console.log(`Loaded ${Object.keys(cache).length} NaPTAN entries from cache`);
    return cache;
  } catch {
    console.log('No NaPTAN cache found locally, fetching from GitHub...');
    // Fetch from GitHub raw content (works on Render where file might not be in working dir)
    try {
      const resp = await fetch('https://raw.githubusercontent.com/TheFirstIstari/papertime/main/static/naptan-cache.json');
      if (resp.ok) {
        const cache = await resp.json();
        console.log(`Fetched ${Object.keys(cache).length} NaPTAN entries from GitHub`);
        // Save locally for future use
        await writeFile(join(STATIC_DIR, 'naptan-cache.json'), JSON.stringify(cache, null, 2));
        return cache;
      }
    } catch (e) {
      console.log('Could not fetch from GitHub: ' + e.message);
    }
    // Try to generate it if Python + naptan are available
    try {
      const { execSync } = await import('child_process');
      const scriptPath = join(__dirname, 'generate_naptan_cache.py');
      execSync(`python3 ${scriptPath}`, { stdio: 'inherit', cwd: join(__dirname, '..') });
      const cache = JSON.parse(await readFile(join(STATIC_DIR, 'naptan-cache.json'), 'utf-8'));
      console.log(`Generated and loaded ${Object.keys(cache).length} NaPTAN entries`);
      return cache;
    } catch (e) {
      console.log('Could not generate NaPTAN cache, using empty mapping');
      return {};
    }
  }
}

async function buildIndex() {
  console.log('Building station index...');
  
  // Read all service files
  const files = (await readdir(SERVICES_DIR)).filter(f => f.endsWith('.json'));
  console.log(`Found ${files.length} service files`);
  
  const stations = new Map();
  
  for (const file of files) {
    const tiploc = file.replace('.json', '');
    const content = await readFile(join(SERVICES_DIR, file), 'utf-8');
    const data = JSON.parse(content);
    
    const services = data.services || [];
    const operators = new Set();
    const destinations = new Set();
    let totalCalls = 0;
    
    for (const svc of services) {
      if (svc.operator) operators.add(svc.operator);
      if (svc.destination_name) destinations.add(svc.destination_name);
      else if (svc.destination) destinations.add(svc.destination);
      totalCalls += (svc.calls || []).length;
    }
    
    // Determine station type
    const naptan = NAptanData[tiploc];
    let stationType = 'minor';
    if (naptan?.type) {
      stationType = naptan.type;
    } else {
      // Heuristic classification
      const name = naptan?.name || tiploc;
      const nameLower = name.toLowerCase();
      if (['euston', 'kings cross', 'paddington', 'waterloo', 'victoria',
           'liverpool street', 'bridge', 'marylebone', 'fenchurch', 'moorgate',
           'stratford', 'canary wharf', 'cambridge', 'oxford', 'birmingham new street',
           'manchester piccadilly', 'leeds', 'glasgow central', 'edinburgh',
           'bristol temple meads', 'cardiff central'].some(n => nameLower.includes(n))) {
        stationType = 'terminal';
      } else if (nameLower.includes('junction') || nameLower.includes('central') ||
                 nameLower.includes('cross') || nameLower.includes('square')) {
        stationType = 'major';
      } else if (name.includes('&') || nameLower.includes(' and ')) {
        stationType = 'interchange';
      }
    }
    
    stations.set(tiploc, {
      id: tiploc,
      name: naptan?.name || tiploc.charAt(0).toUpperCase() + tiploc.slice(1).toLowerCase(),
      tiploc: tiploc,
      lat: naptan?.lat || null,
      lng: naptan?.lng || null,
      type: stationType,
      n_services: services.length,
      operators: [...operators].sort(),
      destinations: [...destinations].sort(),
      file: `services/${file}`
    });
  }
  
  // Convert to array and sort by name
  const index = [...stations.values()].sort((a, b) => a.name.localeCompare(b.name));
  
  // Write output
  await mkdir(dirname(OUTPUT_PATH), { recursive: true });
  await writeFile(OUTPUT_PATH, JSON.stringify(index, null, 2));
  
  console.log(`Wrote ${index.length} stations to ${OUTPUT_PATH}`);
  
  // Stats
  const withCoords = index.filter(s => s.lat).length;
  const byType = {};
  for (const s of index) {
    byType[s.type] = (byType[s.type] || 0) + 1;
  }
  console.log(`  With coordinates: ${withCoords}`);
  console.log(`  By type: ${JSON.stringify(byType)}`);
}

buildIndex().catch(err => {
  console.error('Build failed:', err);
  process.exit(1);
});

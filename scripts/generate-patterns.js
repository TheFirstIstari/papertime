#!/usr/bin/env node
/**
 * generate-patterns.js — Generate service pattern diagrams for each station
 * 
 * Creates static/patterns/{tiploc}.json showing how services diverge:
 * - Groups services by next stop after the station
 * - Shows frequency per destination branch
 * - Includes operator color coding
 * 
 * Usage: node scripts/generate-patterns.js
 */

import { readdir, readFile, writeFile, mkdir } from 'fs/promises';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const STATIC_DIR = join(__dirname, '..', 'static');
const SERVICES_DIR = join(STATIC_DIR, 'services');
const PATTERNS_DIR = join(STATIC_DIR, 'patterns');
const STATION_INDEX_PATH = join(STATIC_DIR, 'station-index.json');

const OP_COLORS = {
  'CC': '#009E73', 'XC': '#009E73', 'SE': '#009E73', 'LE': '#009E73',
  'EM': '#CC79A7', 'GR': '#CC79A7', 'AW': '#CC79A7',
  'LO': '#E86A10', 'ME': '#E86A10',
  'VT': '#E32636', 'HX': '#E32636', 'HT': '#E32636',
  'GW': '#56B4E9', 'SR': '#56B4E9',
  'TP': '#D55E00', 'TL': '#D55E00', 'LM': '#D55E00',
  'NT': '#0072B2', 'SW': '#0072B2', 'CH': '#0072B2',
  'SN': '#F0E442', 'GN': '#F0E442',
  'GC': '#882255', 'GX': '#56B4E9', 'LF': '#E86A10',
  'XR': '#D55E00', 'CS': '#E32636', 'HC': '#0072B2',
  'IL': '#009E73', 'TW': '#D55E00', 'WR': '#56B4E9',
  'XC': '#009E73', 'XP': '#E32636',
};

async function generatePatterns() {
  console.log('Generating service pattern diagrams...');
  
  // Load station index for name lookups
  let stationIndex = [];
  try {
    stationIndex = JSON.parse(await readFile(STATION_INDEX_PATH, 'utf-8'));
  } catch (e) {
    console.log('Warning: Could not load station index');
  }
  
  const stationNames = new Map();
  for (const s of stationIndex) {
    stationNames.set(s.id, s.name);
  }
  
  // Read all service files
  const files = (await readdir(SERVICES_DIR)).filter(f => f.endsWith('.json'));
  console.log(`Processing ${files.length} stations...`);
  
  let generated = 0;
  
  for (const file of files) {
    const tiploc = file.replace('.json', '');
    const content = await readFile(join(SERVICES_DIR, file), 'utf-8');
    const data = JSON.parse(content);
    const services = data.services || [];
    
    if (services.length === 0) continue;
    
    // Group services by their next stop after this station
    const branches = new Map();
    
    for (const svc of services) {
      const calls = svc.calls || [];
      const stationCall = calls.find(c => c.crs === tiploc);
      const stationIdx = calls.indexOf(stationCall);
      
      // Get the next stop after this station
      let nextStop = null;
      if (stationIdx >= 0 && stationIdx < calls.length - 1) {
        nextStop = calls[stationIdx + 1];
      }
      
      // Determine direction from destination
      const dest = svc.destination_name || svc.destination || 'Unknown';
      const destTiploc = svc.destination || '';
      
      // Group by branch (next stop + general direction)
      const branchKey = nextStop ? nextStop.crs : destTiploc;
      const branchName = nextStop ? 
        (stationNames.get(nextStop.crs) || nextStop.crs) : dest;
      
      if (!branches.has(branchKey)) {
        branches.set(branchKey, {
          next_stop: nextStop ? nextStop.crs : null,
          next_stop_name: branchName,
          destination: dest,
          destination_tiploc: destTiploc,
          services: [],
          operators: new Set()
        });
      }
      
      const branch = branches.get(branchKey);
      branch.services.push({
        id: svc.id || svc.headcode,
        operator: svc.operator,
        headcode: svc.headcode,
        dep: stationCall?.dep || null,
        arr: stationCall?.arr || null,
        days: svc.days || ['MF']
      });
      branch.operators.add(svc.operator);
    }
    
    // Convert to output format
    const pattern = {
      station: tiploc,
      station_name: stationNames.get(tiploc) || tiploc,
      n_services: services.length,
      branches: [...branches.values()]
        .sort((a, b) => b.services.length - a.services.length)
        .map(b => ({
          ...b,
          operators: [...b.operators].sort(),
          operator_color: OP_COLORS[[...b.operators][0]] || '#888888',
          frequency: b.services.length,
          services: b.services.sort((a, b) => (a.dep || 0) - (b.dep || 0))
        }))
    };
    
    // Only generate if there are meaningful branches
    if (pattern.branches.length > 0) {
      await mkdir(join(PATTERNS_DIR, tiploc.substring(0, 1).toUpperCase()), { recursive: true });
      await writeFile(
        join(PATTERNS_DIR, `${tiploc}.json`),
        JSON.stringify(pattern, null, 2)
      );
      generated++;
    }
  }
  
  console.log(`Generated ${generated} pattern files`);
}

generatePatterns().catch(err => {
  console.error('Generation failed:', err);
  process.exit(1);
});

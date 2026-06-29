#!/usr/bin/env node
/**
 * Build CRS -> TIPLOC mapping from the actual service files.
 * Each file static/services/{crs}.json contains services that pass through
 * station CRS. The calls use TIPLOC codes. Find which TIPLOC maps to which CRS.
 */

import { readdir, readFile, writeFile } from 'fs/promises';
import { join } from 'path';

const STATIC_DIR = join(process.cwd(), 'static');
const SERVICES_DIR = join(STATIC_DIR, 'services');
const OUTPUT = join(STATIC_DIR, 'crs-to-tiploc.json');

async function build() {
  const files = (await readdir(SERVICES_DIR)).filter(f => f.endsWith('.json'));
  const mapping = {};  // crs -> Set of tiplocs seen

  for (const file of files) {
    const crs = file.replace('.json', '');
    const data = JSON.parse(await readFile(join(SERVICES_DIR, file), 'utf-8'));

    for (const svc of data.services || []) {
      for (const call of svc.calls || []) {
        const tiploc = call.crs;
        if (!mapping[crs]) mapping[crs] = new Set();
        mapping[crs].add(tiploc);
      }
    }
  }

  // Convert sets to arrays - but we want the REVERSE: for each station file,
  // what single TIPLOC code represents THIS station in the calls?
  // Answer: it's the one that appears in EVERY service at the same relative position.
  // Simpler: the file's ID might appear directly, or there's a unique TIPLOC
  // that co-occurs. For now, just output crs -> [all tiplocs seen in its file]
  const output = {};
  for (const [crs, tiplocs] of Object.entries(mapping)) {
    output[crs] = [...tiplocs];
  }

  await writeFile(OUTPUT, JSON.stringify(output));
  console.log(`Wrote mapping for ${Object.keys(output).length} stations`);
}

build();

#!/usr/bin/env node
/**
 * optimize-data.js — Fix service file format and compact all JSON
 *
 * 1. Convert "06:40" string times → 400 integer minutes
 * 2. Compact JSON (no pretty-print) to save ~1.5GB LFS
 * 3. Process services/, marey/, patterns/
 */

import { readdir, readFile, writeFile, stat } from 'fs/promises';
import { join } from 'path';

const STATIC_DIR = join(process.cwd(), 'static');

function timeToMinutes(val) {
  if (val === null || val === undefined) return null;
  if (typeof val === 'number') return val;
  if (typeof val === 'string') {
    const parts = val.split(':');
    if (parts.length < 2) return null;
    const h = parseInt(parts[0], 10);
    const m = parseInt(parts[1], 10);
    if (isNaN(h) || isNaN(m)) return null;
    return h * 60 + m;
  }
  return null;
}

async function processServices() {
  const dir = join(STATIC_DIR, 'services');
  const files = (await readdir(dir)).filter(f => f.endsWith('.json'));
  let fixed = 0, already = 0;

  for (const file of files) {
    const content = await readFile(join(dir, file), 'utf-8');
    const data = JSON.parse(content);
    let modified = false;

    for (const svc of data.services || []) {
      for (const call of svc.calls || []) {
        if (typeof call.arr === 'string') {
          call.arr = timeToMinutes(call.arr);
          modified = true;
        }
        if (typeof call.dep === 'string') {
          call.dep = timeToMinutes(call.dep);
          modified = true;
        }
      }
    }

    // Always rewrite in compact format (saves ~72% space)
    const compact = JSON.stringify(data);
    await writeFile(join(dir, file), compact);

    if (modified) fixed++;
    else already++;
  }

  console.log(`Services: ${already} already correct, ${fixed} fixed times`);
}

async function processMarey() {
  const dir = join(STATIC_DIR, 'marey');
  const files = (await readdir(dir)).filter(f => f.endsWith('.json'));

  for (const file of files) {
    const content = await readFile(join(dir, file), 'utf-8');
    const data = JSON.parse(content);
    // Marey already uses integer minutes, just compact
    const compact = JSON.stringify(data);
    await writeFile(join(dir, file), compact);
  }

  console.log(`Marey: ${files.length} files compacted`);
}

async function processPatterns() {
  const dir = join(STATIC_DIR, 'patterns');
  const files = (await readdir(dir)).filter(f => f.endsWith('.json'));

  let fixed = 0;
  for (const file of files) {
    const content = await readFile(join(dir, file), 'utf-8');
    const data = JSON.parse(content);
    let modified = false;

    for (const branch of data.branches || []) {
      for (const svc of branch.services || []) {
        if (typeof svc.dep === 'string') {
          svc.dep = timeToMinutes(svc.dep);
          modified = true;
        }
        if (typeof svc.arr === 'string') {
          svc.arr = timeToMinutes(svc.arr);
          modified = true;
        }
      }
    }

    const compact = JSON.stringify(data);
    await writeFile(join(dir, file), compact);
    if (modified) fixed++;
  }

  console.log(`Patterns: ${files.length} files compacted, ${fixed} fixed times`);
}

async function reportSavings() {
  const dirs = ['services', 'marey', 'patterns'];
  for (const d of dirs) {
    const dir = join(STATIC_DIR, d);
    const files = (await readdir(dir)).filter(f => f.endsWith('.json'));
    let total = 0;
    for (const f of files) {
      const s = await stat(join(dir, f));
      total += s.size;
    }
    console.log(`  ${d}: ${files.length} files, ${(total / 1024 / 1024).toFixed(1)} MB`);
  }
}

console.log('Before optimization:');
await reportSavings();

console.log('\nProcessing...');
await processServices();
await processMarey();
await processPatterns();

console.log('\nAfter optimization:');
await reportSavings();

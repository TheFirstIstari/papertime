import fs from 'fs';
import path from 'path';

// Load station index
const index = JSON.parse(fs.readFileSync('static/station-index.json', 'utf-8'));

// Build CRS lookup: group stations by name, pick shortest id as CRS
const nameToCrs = {};
for (const s of index) {
  const name = s.name.trim();
  if (!nameToCrs[name] || s.id.length < nameToCrs[name].length) {
    nameToCrs[name] = s.id;
  }
}

// Build TIPLOC→CRS: for stations whose id differs from their name's CRS
const tiplocToCrs = {};
for (const s of index) {
  const crs = nameToCrs[s.name.trim()];
  if (crs && s.id !== crs) {
    tiplocToCrs[s.id] = crs;
  }
}

console.log(`Grouped ${Object.keys(nameToCrs).length} station names, built ${Object.keys(tiplocToCrs).length} TIPLOC→CRS mappings`);

// Show some sample mappings
const samples = Object.entries(tiplocToCrs).slice(0, 10);
for (const [tiploc, crs] of samples) {
  console.log(`  ${tiploc} → ${crs}`);
}

// Fix a services file
function fixServices(data) {
  let fixCount = 0;
  // Skip old-format files (Service, not ServiceRef)
  if (!data.services || !data.services[0] || !data.services[0].calls) return 0;
  for (const svc of data.services) {
    // Fix origin
    if (tiplocToCrs[svc.origin]) {
      svc.origin = tiplocToCrs[svc.origin];
      fixCount++;
    }
    // Fix destination
    if (tiplocToCrs[svc.destination]) {
      svc.destination = tiplocToCrs[svc.destination];
      fixCount++;
    }
    // Fix each call's crs
    for (const call of svc.calls) {
      if (tiplocToCrs[call.crs]) {
        call.crs = tiplocToCrs[call.crs];
        fixCount++;
      }
    }
  }
  return fixCount;
}

// Process all service files
const servicesDir = 'static/services';
let total = 0, totalFixed = 0;
for (const file of fs.readdirSync(servicesDir).filter(f => f.endsWith('.json'))) {
  const data = JSON.parse(fs.readFileSync(path.join(servicesDir, file), 'utf-8'));
  const fixed = fixServices(data);
  if (fixed > 0) {
    fs.writeFileSync(path.join(servicesDir, file), JSON.stringify(data));
    totalFixed += fixed;
  }
  total++;
}
console.log(`\nProcessed ${total} files, fixed ${totalFixed} fields`);
console.log('Done');

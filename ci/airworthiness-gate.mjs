import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');
const SPEC_FILE = path.join(ROOT, 'docs/SPEC.md');

function run() {
  console.log('--- AX-Browser Airworthiness Gate ---');
  
  const specContent = fs.readFileSync(SPEC_FILE, 'utf8');
  const hlrs = [...new Set(specContent.match(/HLR-[0-9]+/g))];
  
  const tracingMarkers = new Map();
  hlrs.forEach(id => tracingMarkers.set(id, false));
  
  const files = getFiles(path.join(ROOT, 'native'))
    .concat(getFiles(path.join(ROOT, 'python')));
  
  for (const file of files) {
    const content = fs.readFileSync(file, 'utf8');
    hlrs.forEach(id => {
      if (content.includes(`Tracing: ${id}`)) tracingMarkers.set(id, true);
    });
  }
  
  let allTraced = true;
  for (const [id, traced] of tracingMarkers) {
    console.log(`${traced ? '✅' : '❌'} ${id}`);
    if (!traced) allTraced = false;
  }
  
  if (!allTraced) process.exit(1);
  console.log('\n✅ 100% Traceability achieved.');
}

function getFiles(dir) {
  let results = [];
  const list = fs.readdirSync(dir);
  list.forEach(file => {
    file = path.join(dir, file);
    const stat = fs.statSync(file);
    if (stat && stat.isDirectory()) results = results.concat(getFiles(file));
    else results.push(file);
  });
  return results;
}

run();

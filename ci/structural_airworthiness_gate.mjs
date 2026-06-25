#!/usr/bin/env node
import { readFileSync, readdirSync, statSync } from 'node:fs';
import { join, relative } from 'node:path';

const phase = process.argv.find(a => a.startsWith('--phase='))?.split('=')[1] || 'all';
const root = process.cwd();
const docsDir = join(root, 'docs');
const srcDir = join(root, 'src');
const schemasDir = join(root, 'schemas');
const generatedDir = join(root, 'ci', 'generated');

const fail = (msg) => { console.error(`FAIL: ${msg}`); process.exit(1); };
const ok = () => { console.log(`OK: ${phase}`); process.exit(0); };

function* walk(dir) {
  for (const entry of readdirSync(dir)) {
    const path = join(dir, entry);
    const st = statSync(path);
    if (st.isDirectory()) yield* walk(path);
    else if (st.isFile()) yield path;
  }
}

function srcFiles() {
  return [...walk(srcDir)].filter(p => p.endsWith('.rs'));
}

function readText(path) {
  return readFileSync(path, 'utf-8');
}

if (phase === 'foundation' || phase === 'all') {
  const requiredDocs = [
    'AXIOMS.md', 'TOPOGRAPHY.md', 'DO178_PLAN.md', 'DEFENSE_IN_DEPTH.md',
    'PERFORMANCE.md', 'OBSERVABILITY.md', 'MACHINE_HUMAN_MEETING.md',
    'JS_WASM_POSITION.md', 'HUMAN_LENS_DEFERRAL.md', 'BRAID_BRIDGE.md',
    'ANTIVIRUS_BROWSER.md', 'LOCKED_FILES.md', 'AIP_INTEGRATION.md'
  ];
  for (const doc of requiredDocs) {
    const path = join(docsDir, doc);
    try { statSync(path); }
    catch { fail(`missing required doc: ${doc}`); }
  }
}

if (phase === 'traceability' || phase === 'all') {
  const axiomText = readText(join(docsDir, 'AXIOMS.md'));
  for (const file of srcFiles()) {
    const rel = relative(root, file);
    if (rel.includes('mod.rs')) continue;
    const text = readText(file);
    const hasAxiomRef = /AXIOM_/.test(text) || text.includes('browser_axioms');
    if (!hasAxiomRef) fail(`${rel} has no axiom traceability reference`);
  }
}

if (phase === 'docs' || phase === 'all') {
  for (const file of [...walk(docsDir)]) {
    if (!file.endsWith('.md')) continue;
    const text = readText(file);
    if (!text.includes('##')) fail(`${relative(root, file)} missing section headers`);
  }
}

if (phase === 'vocab' || phase === 'all') {
  const allowedVerbs = new Set(JSON.parse(readText(join(generatedDir, 'web_action_verbs.json'))));
  const termFamilies = new Set([
    'web.element', 'web.observation', 'web.action', 'web.capability',
    'web.verdict', 'web.transition', 'web.obs.aip_state', 'web.obs.aip_policy',
    'web.act.aip_action', 'web.cap.aip_delegation'
  ]);
  for (const file of srcFiles()) {
    const text = readText(file);
    const matches = [...text.matchAll(/web\.[a-z_]+(?:\.[a-z_]+)?/g)].map(m => m[0]);
    for (const m of matches) {
      if (termFamilies.has(m)) continue;
      if (!allowedVerbs.has(m)) fail(`${relative(root, file)} uses unapproved action verb ${m}`);
    }
  }
}

if (phase === 'dup' || phase === 'all') {
  const names = new Map();
  for (const file of srcFiles()) {
    const base = file.split('/').pop();
    if (base === 'mod.rs') continue;
    if (names.has(base)) fail(`duplicate file name: ${base}`);
    names.set(base, file);
  }
}

if (phase === 'audit-struct' || phase === 'all') {
  const locked = readText(join(docsDir, 'LOCKED_FILES.md'));
  for (const file of srcFiles()) {
    const rel = relative(root, file);
    if (!locked.includes(rel)) fail(`${rel} not listed in LOCKED_FILES.md`);
  }
}

if (phase === 'api-coverage' || phase === 'all') {
  const lib = readText(join(srcDir, 'lib.rs'));
  for (const mod of ['capability', 'policy', 'tape', 'observation', 'action', 'state_machine', 'boundary', 'platform', 'compute', 'audit', 'braid_bridge']) {
    if (!lib.includes(`pub mod ${mod};`)) fail(`missing pub mod ${mod} in lib.rs`);
  }
}

if (phase === 'forbidden-api' || phase === 'all') {
  const forbidden = ['std::fs', 'std::net', 'std::process', 'std::env', 'unsafe'];
  for (const file of srcFiles()) {
    const text = readText(file);
    for (const pat of forbidden) {
      if (text.includes(pat)) fail(`${relative(root, file)} contains forbidden API: ${pat}`);
    }
  }
}

if (phase === 'capability-reach' || phase === 'all') {
  const broker = readText(join(srcDir, 'capability', 'mod.rs'));
  if (!broker.includes('attenuation')) fail('capability broker missing attenuation concept');
}

if (phase === 'origin-policy' || phase === 'all') {
  const policy = readText(join(srcDir, 'boundary', 'url_policy.rs'));
  if (!policy.includes('deny-first')) fail('url_policy.rs missing deny-first doctrine');
}

if (phase === 'prompt-inject' || phase === 'all') {
  const policy = readText(join(srcDir, 'policy', 'broker.rs'));
  const codeOnly = policy.split('\n').filter(l => !l.trim().startsWith('//') && !l.trim().startsWith('*')).join('\n');
  if (/\bllm\b/i.test(codeOnly)) {
    fail('policy broker must not reference LLM in code');
  }
}

if (phase === 'schema' || phase === 'all') {
  const required = [
    'web_anchor.json', 'web_action.json', 'web_observation.json', 'web_capability.json', 'web_tape.json',
    'aip_policy.json', 'aip_state.json', 'aip_delegation.json', 'aip_privacy.json'
  ];
  for (const s of required) {
    try { statSync(join(schemasDir, s)); }
    catch { fail(`missing schema ${s}`); }
  }
}

if (phase === 'mcdc' || phase === 'all') {
  const table = readText(join(srcDir, 'state_machine', 'transition_table.rs'));
  if (!table.includes('Verdict')) fail('transition table does not consume Verdict');
}

if (phase === 'frame-budget' || phase === 'all') {
  const perf = readText(join(docsDir, 'PERFORMANCE.md'));
  if (!perf.includes('16 ms')) fail('PERFORMANCE.md missing 16 ms frame budget');
}

if (phase === 'alloc-budget' || phase === 'all') {
  const perf = readText(join(docsDir, 'PERFORMANCE.md'));
  if (!/zero[-\s]?allocation/i.test(perf)) fail('PERFORMANCE.md missing zero-allocation hot path');
}

ok();

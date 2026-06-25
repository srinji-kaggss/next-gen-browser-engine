#!/usr/bin/env node
// Canonical generator for the "Rust Source Seams" section of docs/LOCKED_FILES.md.
//
// The structural airworthiness gate (audit-struct phase) requires every src/**/*.rs
// file to be listed in LOCKED_FILES.md. That list is a mechanical mirror of the
// filesystem, not curated governance prose, so it must be GENERATED rather than
// hand-edited. This module is the single source of truth for that computation;
// the gate imports computeSrcManifest() from here so generation and enforcement
// can never drift apart.
//
// Usage:
//   node ci/gen_locked_files.mjs           # rewrite the section in place
//   node ci/gen_locked_files.mjs --check   # exit 1 if the section is out of date
import { readFileSync, writeFileSync, readdirSync, statSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { join, relative, sep } from 'node:path';

const SECTION_HEADER = '## Rust Source Seams';

function* walk(dir) {
  for (const entry of readdirSync(dir)) {
    const path = join(dir, entry);
    const st = statSync(path);
    if (st.isDirectory()) yield* walk(path);
    else if (st.isFile()) yield path;
  }
}

/**
 * The canonical, deterministic set of locked Rust source seams: every .rs file
 * under src/, as POSIX-style repo-relative paths, sorted lexicographically.
 * This MUST stay identical to the gate's srcFiles() set.
 */
export function computeSrcManifest(root) {
  const srcDir = join(root, 'src');
  return [...walk(srcDir)]
    .filter(p => p.endsWith('.rs'))
    .map(p => relative(root, p).split(sep).join('/'))
    .sort();
}

/** Render the manifest as the body lines of the section. */
export function renderManifestBlock(manifest) {
  return manifest.map(p => `- \`${p}\``).join('\n');
}

/** Extract the existing body of the Rust Source Seams section (between its
 *  header and the next "## " header, trimmed). Returns null if absent. */
export function extractSeamsSection(text) {
  const lines = text.split('\n');
  const start = lines.findIndex(l => l.trim() === SECTION_HEADER);
  if (start === -1) return null;
  let end = lines.length;
  for (let i = start + 1; i < lines.length; i++) {
    if (lines[i].startsWith('## ')) { end = i; break; }
  }
  return lines.slice(start + 1, end).join('\n').trim();
}

/** Splice a freshly generated section body into the document, preserving the
 *  surrounding blank-line spacing of the original section. */
function spliceSection(text, block) {
  const lines = text.split('\n');
  const start = lines.findIndex(l => l.trim() === SECTION_HEADER);
  if (start === -1) throw new Error(`section not found: ${SECTION_HEADER}`);
  let end = lines.length;
  for (let i = start + 1; i < lines.length; i++) {
    if (lines[i].startsWith('## ')) { end = i; break; }
  }
  // Keep trailing blank lines that belonged to the section (before next header).
  let tailBlanks = 0;
  for (let i = end - 1; i > start && lines[i].trim() === ''; i--) tailBlanks++;
  const rebuilt = [
    ...lines.slice(0, start + 1),
    '',
    block,
    ...Array(tailBlanks).fill(''),
    ...lines.slice(end),
  ];
  return rebuilt.join('\n');
}

function main() {
  const root = process.cwd();
  const lockedPath = join(root, 'docs', 'LOCKED_FILES.md');
  const text = readFileSync(lockedPath, 'utf-8');
  const expected = renderManifestBlock(computeSrcManifest(root));
  const actual = extractSeamsSection(text);
  const check = process.argv.includes('--check');

  if (actual === expected) {
    if (check) console.log('OK: Rust Source Seams manifest up to date');
    return;
  }
  if (check) {
    console.error('FAIL: docs/LOCKED_FILES.md Rust Source Seams section is stale.');
    console.error('Run: node ci/gen_locked_files.mjs');
    process.exit(1);
  }
  writeFileSync(lockedPath, spliceSection(text, expected));
  console.log('regenerated: docs/LOCKED_FILES.md Rust Source Seams');
}

if (process.argv[1] === fileURLToPath(import.meta.url)) main();

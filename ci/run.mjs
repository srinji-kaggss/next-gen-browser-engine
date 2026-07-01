#!/usr/bin/env node
import { readFileSync } from 'node:fs';
import { execSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const registryPath = join(__dirname, 'registry.json');
const registry = JSON.parse(readFileSync(registryPath, 'utf-8'));

let failed = 0;
let passed = 0;

for (const verifier of registry.verifiers) {
  process.stdout.write(`${verifier.id} ... `);
  try {
    execSync(verifier.command, { stdio: 'pipe', cwd: process.cwd() });
    process.stdout.write('PASS\n');
    passed++;
  } catch (err) {
    process.stdout.write('FAIL\n');
    if (err.stderr) process.stderr.write(err.stderr.toString());
    if (err.stdout) process.stderr.write(err.stdout.toString());
    failed++;
  }
}

process.stdout.write(`\n${passed} passed, ${failed} failed\n`);
if (failed > 0) process.exit(1);

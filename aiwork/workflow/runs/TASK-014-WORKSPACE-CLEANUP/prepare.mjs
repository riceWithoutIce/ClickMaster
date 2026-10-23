// Preparation only: inventory, hashes and reviewable plan. No deletion or moves.
import fs from 'node:fs';
import path from 'node:path';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import { execFileSync } from 'node:child_process';
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../../../..');
const recordRoot = 'aiwork/workflow/runs/TASK-014-WORKSPACE-CLEANUP';
const planRoot = 'build/verification/TASK-014-WORKSPACE-CLEANUP/preparation';
const latest = 'build/staging/TASK-013-UI-SKETCH-ALIGN';
const oldManifestPath = 'build/verification/TASK-013-UI-SKETCH-ALIGN/coder/source-manifest.json';
const now = new Date().toISOString();
const tracked = new Set(execFileSync('git', ['ls-files', '-z'], { cwd: root, encoding: 'utf8', windowsHide: true }).split('\0').filter(Boolean));
const abs = rel => {
  const resolved = path.resolve(root, rel);
  if (!resolved.startsWith(root + path.sep)) throw new Error('Path outside repository');
  return resolved;
};
function walk(rel, files = [], links = []) {
  if (rel === 'build/verification/TASK-014-WORKSPACE-CLEANUP') return { files, links };
  if (!fs.existsSync(abs(rel))) return { files, links };
  for (const entry of fs.readdirSync(abs(rel), { withFileTypes: true })) {
    const name = `${rel}/${entry.name}`;
    const s = fs.lstatSync(abs(name));
    if (s.isSymbolicLink()) { links.push(name); continue; }
    if (s.isDirectory()) walk(name, files, links);
    else if (s.isFile()) files.push({ path: name, bytes: s.size, mtimeMs: s.mtimeMs });
  }
  return { files, links };
}
async function hash(rel) {
  const h = createHash('sha256');
  if (fs.statSync(abs(rel)).size <= 32 * 1024 * 1024) return h.update(fs.readFileSync(abs(rel))).digest('hex').toUpperCase();
  for await (const chunk of fs.createReadStream(abs(rel), { highWaterMark: 4 * 1024 * 1024 })) h.update(chunk);
  return h.digest('hex').toUpperCase();
}
function write(name, value) {
  const destination = name === 'file-plan.json' ? planRoot : recordRoot;
  fs.mkdirSync(abs(destination), { recursive: true });
  fs.writeFileSync(abs(`${destination}/${name}`), JSON.stringify(value, null, 2) + '\n');
}
const generated = new Set(['node_modules', 'target', 'build', '.git', 'node-cache', 'cache', 'tmp', 'cargo-home', 'optimized-target', 'optimized-final-target']);
const binary = new Set(['.exe', '.zip', '.pdb', '.dll', '.rlib', '.rmeta', '.lib', '.obj', '.o', '.exp']);
function classify(item) {
  const rel = item.path;
  if (tracked.has(rel) || path.basename(rel) === '.gitkeep') return { action: 'keep-evidence', reason: 'tracked-file-or-directory-contract' };
  if (rel.startsWith(latest + '/')) return { action: 'keep-active-current', reason: 'latest-accepted-rc9-and-live-runtime' };
  const parts = rel.split('/');
  if (parts.slice(2).some(p => ['webview', 'webview2', 'ebwebview'].includes(p.toLowerCase()))) return { action: 'delete', reason: 'historical-webview-cache' };
  if (rel.startsWith('build/tmp/')) return { action: 'delete', reason: 'project-temporary-output' };
  if (rel.startsWith('build/staging/')) {
    if (parts.includes('data')) return { action: 'keep-evidence', reason: 'historical-config-logs-or-runtime-receipts' };
    return { action: 'delete', reason: 'obsolete-candidate-after-version-record' };
  }
  if (parts.slice(2).some(p => generated.has(p.toLowerCase()))) return { action: 'delete', reason: 'isolated-regenerable-build-or-dependency-cache' };
  if (parts.slice(2).includes('source')) return { action: 'delete', reason: 'historical-full-source-copy-current-source-and-reports-retained' };
  if (binary.has(path.extname(rel).toLowerCase())) return { action: 'delete', reason: 'historical-executable-package-or-debug-output' };
  if (path.extname(rel).toLowerCase() === '.bin') {
    const synthetic = /^build\/verification\/TASK-008-IPC-EFFICIENCY\/performance\/(harness|optimized-harness|optimized-final-harness)\/fixture-assembly\.bin$/.test(rel)
      && item.bytes === 4194304 && fs.readFileSync(abs(rel)).every(x => x === 0x5a);
    return synthetic ? { action: 'delete', reason: 'regenerable-synthetic-performance-fixture' } : { action: 'hold-review', reason: 'binary-fixture-purpose-needs-check' };
  }
  return { action: 'keep-evidence', reason: 'report-log-manifest-fixture-script-or-ui-reference' };
}
fs.mkdirSync(abs(recordRoot), { recursive: true });
const inventory = [], links = [];
for (const rel of ['build/staging', 'build/verification', 'build/tmp']) {
  const found = walk(rel);
  inventory.push(...found.files); links.push(...found.links);
}
if (fs.existsSync(abs('0.1.0-rc.8'))) {
  const text = fs.readFileSync(abs('0.1.0-rc.8'), 'utf8').trim();
  const s = fs.statSync(abs('0.1.0-rc.8'));
  inventory.push({ path: '0.1.0-rc.8', bytes: s.size, mtimeMs: s.mtimeMs, strayConfirmed: text === 'version/VERSION' });
}
const manifest = [];
for (const item of inventory) {
  const classification = item.path === '0.1.0-rc.8'
    ? { action: item.strayConfirmed ? 'delete' : 'hold-review', reason: 'root-version-command-residue' }
    : classify(item);
  // Active runtime files may change during collection. They are never deletion candidates.
  const digest = classification.action === 'keep-active-current' && item.path.includes('/data/') ? null : await hash(item.path);
  manifest.push({ path: item.path, bytes: item.bytes, sha256: digest, ...classification });
}
const groups = {};
for (const item of manifest) {
  const k = `${item.action}:${item.reason}`;
  groups[k] ??= { action: item.action, reason: item.reason, files: 0, bytes: 0 };
  groups[k].files++; groups[k].bytes += item.bytes;
}
const versions = [];
for (const dir of fs.readdirSync(abs('build/staging'), { withFileTypes: true }).filter(x => x.isDirectory())) {
  const rel = `build/staging/${dir.name}`;
  const sourcePath = `${rel}/source.json`;
  const source = fs.existsSync(abs(sourcePath)) ? JSON.parse(fs.readFileSync(abs(sourcePath), 'utf8')) : null;
  const artifacts = manifest.filter(x => x.path.startsWith(rel + '/') && ['.exe', '.zip'].includes(path.extname(x.path).toLowerCase()) && !x.path.includes('/data/'));
  const taskDir = `${recordRoot.substring(0, recordRoot.lastIndexOf('/'))}/${dir.name}`;
  const baseTask = dir.name === 'TASK-006-UI-CLEANUP-rc2' ? 'aiwork/workflow/runs/TASK-006-UI-CLEANUP' : taskDir;
  const acceptanceRecords = fs.existsSync(abs(baseTask)) ? fs.readdirSync(abs(baseTask)).filter(n => /human|acceptance|verifier|coder-handoff/.test(n)).map(n => `${baseTask}/${n}`) : [];
  const expected = source ? ['exe', 'zip'].filter(k => source[k]?.sha256).map(k => {
    const declared = source[k];
    const actual = manifest.find(a => a.path === declared.path);
    return { kind: k, declaredPath: declared.path, declaredSha256: declared.sha256, matches: actual ? actual.sha256.toUpperCase() === declared.sha256.toUpperCase() : null };
  }) : [];
  const sourceMetadata = source ? Object.fromEntries(['id', 'version', 'headReferenceOnly', 'dirtySnapshot', 'sourceFileCount', 'sourceManifest', 'sourceManifestSha256', 'signature', 'release'].filter(k => k in source).map(k => [k, source[k]])) : null;
  versions.push({ directory: rel, version: source?.version ?? artifacts.map(a => a.path.match(/ClickMaster-(.+)-windows-x64\.zip$/)?.[1]).find(Boolean) ?? 'see-task-records', published: false, status: rel === latest ? 'keep-current-accepted' : 'planned-removal-not-executed', sourceMetadata, artifacts, declaredChecks: expected, acceptanceRecords, sourceRecord: sourcePath, sourceRecordSha256: fs.existsSync(abs(sourcePath)) ? await hash(sourcePath) : null });
}
const baseline = JSON.parse(fs.readFileSync(abs(oldManifestPath), 'utf8'));
const protectedSources = [];
for (const item of baseline) protectedSources.push({ path: item.path, bytes: fs.statSync(abs(item.path)).size, sha256: await hash(item.path), acceptedSnapshotSha256: item.sha256, changedSinceAcceptedSnapshot: (await hash(item.path)) !== item.sha256 });
const shared = [];
for (const rel of ['build/cargo', 'build/cache', 'build/frontend', 'build/logs', 'node_modules', 'src-tauri/gen']) {
  const found = walk(rel);
  shared.push({ path: rel, action: 'keep-until-release-package-complete', files: found.files.length, bytes: found.files.reduce((sum, x) => sum + x.bytes, 0) });
}
const summary = { preparedAt: now, mode: 'prepare-only-no-deletion-no-move', rule: 'retain-latest-accepted-binaries-history-metadata-only', latestAcceptedCandidate: latest, activeApplicationObservation: 'rc9 application instances observed; keep entire rc9 runtime; recheck processes before execution; no process terminated', candidates: manifest.filter(x => x.action === 'delete').length, candidateBytes: manifest.filter(x => x.action === 'delete').reduce((s, x) => s + x.bytes, 0), grouped: Object.values(groups), deferredSharedOutputs: shared, linksHeld: links, removalExecuted: false };
write('file-plan.json', { preparedAt: now, relativePathsOnly: true, entries: manifest });
write('historical-versions.json', { preparedAt: now, removalExecuted: false, versions });
write('protected-sources.json', { preparedAt: now, note: 'Documentation changes after rc9 acceptance are expected and are not binary drift.', entries: protectedSources });
write('summary.json', summary);
console.log(JSON.stringify({ ...summary, grouped: summary.grouped, versions: versions.map(v => ({ version: v.version, status: v.status, artifacts: v.artifacts.length, declaredChecks: v.declaredChecks })) }, null, 2));

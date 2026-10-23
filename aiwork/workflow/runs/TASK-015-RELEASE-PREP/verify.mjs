// Coordinator checks of a frozen package; no GUI, real input or publication.
import fs from 'node:fs';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { resolve } from 'node:path';
import { root } from '../../../../scripts/environment.mjs';
import { inspectPe } from '../../../../scripts/inspect-pe.mjs';
import { inspectIcon } from '../../../../scripts/verify-pe-icon.mjs';
const out='build/verification/TASK-015-RELEASE-PREP/coordinator';
const stage='build/staging/TASK-015-RELEASE-PREP';
const read=p=>fs.readFileSync(resolve(root,p));
const load=p=>JSON.parse(read(p).toString('utf8').replace(/^\uFEFF/,''));
const sha=b=>createHash('sha256').update(b).digest('hex').toUpperCase();
const source=load(`${stage}/source.json`);
const manifest=load(source.sourceManifest);
assert.equal(sha(read(source.sourceManifest)),source.sourceManifestSha256);
for(const entry of manifest) { assert.equal(read(entry.path).length,entry.bytes); assert.equal(sha(read(entry.path)),entry.sha256,entry.path); }
assert.equal(sha(read(source.zip.path)),source.zip.sha256);
assert.equal(sha(read(source.executable.path)),source.executable.sha256);
const pe=inspectPe(read(source.executable.path));assert.equal(pe.machine,'AMD64');
assert.ok(inspectIcon(read(source.executable.path),read('src-tauri/icons/icon.ico')).matchesExpected);
const old=load('build/staging/TASK-013-UI-SKETCH-ALIGN/source.json');
assert.equal(sha(read(old.exe.path)),old.exe.sha256);assert.equal(sha(read(old.zip.path)),old.zip.sha256);
const git=(...args)=>execFileSync('git',args,{cwd:root,encoding:'utf8',windowsHide:true});
assert.equal(git('diff','--name-only','HEAD','--','src','src-tauri/src','scripts').trim(),'','Production behavior source changed');
for(const file of ['package.json','package-lock.json','src-tauri/tauri.conf.json']) {
 const current=load(file), baseline=JSON.parse(git('show',`HEAD:${file}`));
 current.version=baseline.version;
 if(file==='package-lock.json')current.packages[''].version=baseline.packages[''].version;
 assert.deepEqual(current,baseline,`${file}: non-version changes`);
}
for(const file of ['src-tauri/Cargo.toml','src-tauri/Cargo.lock']) {
 const current=read(file).toString().replaceAll('\r\n','\n'),baseline=git('show',`HEAD:${file}`).replaceAll('\r\n','\n');
 const regex=file.endsWith('.lock')?/(name = "clickmaster"\nversion = ")([^"\n]+)(")/:/(\[package\][\s\S]*?\nversion = ")([^"\n]+)(")/;
 assert.equal(current.replace(regex,(_,a,b,c)=>a+'0.1.0-rc.9'+c),baseline);
}
const known=[root,root.replaceAll('\\','/'),process.env.USERPROFILE].filter(Boolean);
const textPaths=[...manifest.map(e=>e.path),`${stage}/README.md`,`${stage}/source.json`,`${stage}/SHA256SUMS.txt`,...fs.readdirSync(resolve(root,'aiwork/workflow/runs/TASK-015-RELEASE-PREP')).filter(n=>/\.(md|ps1|mjs|json)$/.test(n)).map(n=>`aiwork/workflow/runs/TASK-015-RELEASE-PREP/${n}`)];
let texts=0;const prefixHits=[],credentialHits=[];
for(const path of textPaths) {
 const bytes=read(path);if(bytes.includes(0))continue;texts++;const text=bytes.toString('utf8');
 if(known.some(k=>text.toLowerCase().includes(k.toLowerCase())))prefixHits.push(path);
 if(/-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----|gh[pousr]_[A-Za-z0-9]{20,}|github_pat_[A-Za-z0-9_]{30,}|sk-[A-Za-z0-9_-]{24,}/.test(text))credentialHits.push(path);
}
const binary=read(source.executable.path);
for(const p of known) for(const encoding of ['utf8','utf16le'])assert.equal(binary.includes(Buffer.from(p,encoding)),false,'Known machine prefix embedded');
assert.deepEqual(prefixHits,[]);assert.deepEqual(credentialHits,[]);
const results=load(`${out}/results.json`);assert.ok(results.every(r=>r.exit===0));
const backend=read(`${out}/backend.log`).toString();assert.match(backend,/100 passed; 0 failed; 2 ignored/);
assert.deepEqual(load('version/releases.json').releases,[],'Release index was promoted before acceptance');
assert.equal(source.release,false);assert.equal(source.humanAcceptance,'NOT RUN');
const report={version:'0.1.0',sourceFiles:manifest.length,sourceAndPackageHashes:'PASS',unchangedBehaviorSource:'PASS',dependenciesUnchangedExceptOwnVersion:'PASS',pe,peIcon:'PASS',preservedRc9:'PASS',textFilesScanned:texts,knownPrefixScan:'PASS',commonCredentialPatternScan:'PASS',scanScope:'Bounded automated scan; not exhaustive privacy proof',backend:{rawEntry:'FAILED-TO-START-0xc0000139',isolatedCommonControlsManifest:'100-passed-0-failed-2-ignored'},coordinatorChecks:'PASS',independentVerifier:'NOT RUN',humanAcceptance:'NOT RUN',windows11:'NOT RUN',missingOrOldRuntime:'NOT RUN',release:false};
fs.writeFileSync(resolve(root,`${out}/closure.json`),JSON.stringify(report,null,2)+'\n');
console.log(JSON.stringify(report,null,2));

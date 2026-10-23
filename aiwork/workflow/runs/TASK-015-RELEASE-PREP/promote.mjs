// One-time, authorized local promotion. Never rebuild or launch the accepted package.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {root} from '../../../../scripts/environment.mjs';
const task='aiwork/workflow/runs/TASK-015-RELEASE-PREP';
const staging='build/staging/TASK-015-RELEASE-PREP';
const current='release/current';
const zip='ClickMaster-0.1.0-windows-x64.zip';
const absolute=rel=>{
 const p=path.resolve(root,rel);assert.ok(p.startsWith(root+path.sep));return p;
};
const read=rel=>fs.readFileSync(absolute(rel));
const load=rel=>JSON.parse(read(rel).toString('utf8').replace(/^\uFEFF/,''));
const hash=rel=>createHash('sha256').update(read(rel)).digest('hex').toUpperCase();
const write=(rel,value)=>fs.writeFileSync(absolute(rel),JSON.stringify(value,null,2)+'\n');
const source=load(`${staging}/source.json`);
assert.equal(hash(`${staging}/${zip}`),'ACEE1AE8045055D01DECE59F56FE7D238FA7A3DDEB9F2C6B2C9447EF008630DD');
assert.equal(hash(source.executable.path),'9F263B8AC713DDDEF93C929DA38A6804077BAD0FAD82FD21B012894EE0A5DE88');
assert.equal(hash(source.sourceManifest),source.sourceManifestSha256);
assert.match(read(`${task}/human-acceptance.md`).toString(),/用户人工确认通过/);
assert.deepEqual(fs.readdirSync(absolute(current)),['README.md']);
assert.deepEqual(load('version/releases.json').releases,[]);
const manifest=load(source.sourceManifest),docChanges=[];
for(const e of manifest){
 if(hash(e.path)!==e.sha256){
  assert.ok(/^(README\.md|docs\/.*\.md|release\/.*\.md|version\/(CHANGELOG|README)\.md)$/.test(e.path),`Unexpected post-build change: ${e.path}`);
  docChanges.push(e.path);
 }
}
fs.mkdirSync(absolute('version/manifests'),{recursive:true});
const durableManifest='version/manifests/0.1.0-build.json';
assert.equal(fs.existsSync(absolute(durableManifest)),false);
write(durableManifest,manifest);
const buildSnapshot={...source,sourceManifest:durableManifest,sourceManifestSha256:hash(durableManifest),originalSourceManifest:source.sourceManifest,originalSourceManifestSha256:source.sourceManifestSha256};
write(`${task}/accepted-build-source.json`,buildSnapshot);
// Rename the exact absolute file within the same workspace; no directory recursion.
assert.equal(fs.existsSync(absolute(`${current}/${zip}`)),false);
fs.renameSync(absolute(`${staging}/${zip}`),absolute(`${current}/${zip}`));
assert.equal(hash(`${current}/${zip}`),source.zip.sha256);
fs.writeFileSync(absolute(`${current}/SHA256SUMS.txt`),`${source.executable.sha256}  ClickMaster.exe\n${source.zip.sha256}  ${zip}\n`);
const entry={
 version:'0.1.0',channel:'stable',date:'2026-10-23',architecture:'x64',format:'portable-zip',
 sourceRef:'refs/tags/v0.1.0',buildBaseCommit:source.baseCommit,buildDirtySnapshot:true,
 buildManifest:durableManifest,buildManifestSha256:hash(durableManifest),
 buildFileHashMode:source.sourceTextHashMode,postBuildDocumentationChanges:docChanges,
 artifact:{filename:zip,path:`${current}/${zip}`,bytes:source.zip.bytes,sha256:source.zip.sha256},
 executable:{packageEntry:'ClickMaster.exe',bytes:source.executable.bytes,sha256:source.executable.sha256},
 runtime:'Compatible shared Microsoft Evergreen WebView2 required; not bundled',signature:'NotSigned',
 acceptance:{human:'PASS-user-confirmed',record:`${task}/human-acceptance.md`,preparation:`${task}/preparation-report.md`,backend:'100 passed / 0 failed / 2 ignored in isolated test copy with Common Controls v6 manifest',rawBackendEntry:'FAILED-TO-START-0xc0000139',independentVerifier:'NOT RUN',windows11:'NOT RUN',missingOrOldRuntime:'NOT RUN',longTermPerformance:'NOT RUN'},
 retention:{status:'current'},publication:{state:'local-promoted-remote-pending',tag:'v0.1.0',intendedUrl:'https://github.com/riceWithoutIce/ClickMaster/releases/tag/v0.1.0'}
};
write('version/releases.json',{schemaVersion:1,releases:[entry]});
write(`${task}/promotion-result.json`,{version:'0.1.0',status:'local-promoted-remote-pending',artifact:entry.artifact,buildManifest:durableManifest,buildManifestSha256:hash(durableManifest),postBuildDocumentationChanges:docChanges,acceptedZipUnchanged:true,stagingZipRemoved:true,currentRuntimePreserved:true,commit:false,tag:false,githubRelease:false});
console.log(JSON.stringify({status:'local-promoted',artifact:entry.artifact,postBuildDocumentationChanges:docChanges},null,2));

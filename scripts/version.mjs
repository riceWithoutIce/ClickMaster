// VERSION is the only edited version source. Sync changes own metadata, never dependency versions.
import {readFileSync,writeFileSync} from 'node:fs';
import {root} from './environment.mjs';
import {resolve} from 'node:path';
const version=readFileSync(resolve(root,'version/VERSION'),'utf8').trim();
if(!/^\d+\.\d+\.\d+(?:-(?:dev|rc|alpha|beta)\.\d+)?$/.test(version))throw new Error('Unsupported version');
const sync=process.argv.includes('--sync');
let mismatches=[];
for(const path of ['package.json','package-lock.json','src-tauri/tauri.conf.json']) {
 const file=resolve(root,path), source=readFileSync(file,'utf8');const data=JSON.parse(source);
 if(data.version!==version || (path==='package-lock.json' && data.packages[''].version!==version)) {
  mismatches.push(path); if(sync){const ownVersion=data.version;let next=source.replace('"version": "'+ownVersion+'"','"version": "'+version+'"');if(path==='package-lock.json')next=next.replace('"version": "'+data.packages[''].version+'"','"version": "'+version+'"');writeFileSync(file,next);}
 }
}
for(const path of ['src-tauri/Cargo.toml','src-tauri/Cargo.lock']) {
 const file=resolve(root,path),source=readFileSync(file,'utf8');const regex=path.endsWith('.lock')?/(name = "clickmaster"\r?\nversion = ")([^"\n]+)(")/:/(\[package\][\s\S]*?\nversion = ")([^"\n]+)(")/;
 const match=source.match(regex);if(!match)throw new Error('Own package metadata missing');if(match[2]!==version){mismatches.push(path);if(sync)writeFileSync(file,source.replace(regex,(_,a,b,c)=>a+version+c));}
}
if(mismatches.length && !sync)throw new Error('VERSION mismatch: '+mismatches.join(', '));
console.log('Version '+version+': '+(sync?'synchronized own metadata':'PASS'));

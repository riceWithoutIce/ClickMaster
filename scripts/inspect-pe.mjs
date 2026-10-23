// Read-only import/architecture inspection, without loading the inspected executable.
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {fileURLToPath} from 'node:url';
export function inspectPe(bytes) {
  const u16=o=>bytes.readUInt16LE(o),u32=o=>bytes.readUInt32LE(o);
  if(u16(0)!==0x5a4d)throw new Error('Missing MZ');
  const pe=u32(0x3c);if(u32(pe)!==0x4550)throw new Error('Missing PE');
  const opt=pe+24,magic=u16(opt);if(magic!==0x20b)throw new Error('Expected PE32+');
  const sections=[];
  for(let i=0;i<u16(pe+6);i++) {const at=opt+u16(pe+20)+i*40;sections.push({rva:u32(at+12),size:u32(at+16),offset:u32(at+20)});}
  function file(rva) {const s=sections.find(s=>rva>=s.rva&&rva<s.rva+s.size);if(!s)throw new Error('Unmapped RVA');return s.offset+rva-s.rva;}
  function name(rva) {const at=file(rva),end=bytes.indexOf(0,at);if(end<at||end-at>256)throw new Error('Invalid import name');return bytes.toString('ascii',at,end);}
  const directories=opt+112;
  function imports(index,stride,nameOffset) {
    const rva=u32(directories+index*8),size=u32(directories+index*8+4);if(!rva)return [];
    const at=file(rva),result=[];
    for(let off=0;off+stride<=size;off+=stride) {
      if(bytes.subarray(at+off,at+off+stride).every(v=>v===0))return result;
      if(index===13&&!(u32(at+off)&1))throw new Error('Unexpected VA delay import');
      result.push(name(u32(at+off+nameOffset)));
    }
    throw new Error('Unterminated import table');
  }
  return {machine:u16(pe+4)===0x8664?'AMD64':u16(pe+4).toString(16),pe32plus:true,imports:[...new Set(imports(1,20,12))].sort(),delayImports:[...new Set(imports(13,32,4))].sort()};
}
if(process.argv[1]&&resolve(process.argv[1])===fileURLToPath(import.meta.url))console.log(JSON.stringify(inspectPe(readFileSync(process.argv[2])),null,2));

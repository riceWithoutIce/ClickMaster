// Offline PE resource reader: never loads or executes the inspected program.
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const hash = bytes => createHash('sha256').update(bytes).digest('hex');
function view(bytes) {
  function range(offset, length) {
    if (!Number.isSafeInteger(offset) || offset < 0 || length < 0 || offset + length > bytes.length) throw new Error('Invalid binary range');
    return bytes.subarray(offset, offset + length);
  }
  return { range, u16: offset => range(offset, 2).readUInt16LE(), u32: offset => range(offset, 4).readUInt32LE() };
}

export function inspectIcon(exeBytes, icoBytes) {
  const pe = view(exeBytes);
  if (pe.u16(0) !== 0x5a4d) throw new Error('Missing MZ signature');
  const header = pe.u32(0x3c);
  if (pe.u32(header) !== 0x4550) throw new Error('Missing PE signature');
  const optional = header + 24;
  const magic = pe.u16(optional);
  if (![0x10b, 0x20b].includes(magic)) throw new Error('Unsupported PE optional header');
  const directories = optional + (magic === 0x20b ? 112 : 96);
  const resourceRva = pe.u32(directories + 16);
  const resourceSize = pe.u32(directories + 20);
  const sections = [];
  const sectionTable = optional + pe.u16(header + 20);
  for (let i = 0; i < pe.u16(header + 6); i++) {
    const offset = sectionTable + i * 40;
    sections.push({ address: pe.u32(offset + 12), size: pe.u32(offset + 16), file: pe.u32(offset + 20) });
  }
  function rvaToFile(rva, length) {
    const section = sections.find(s => rva >= s.address && rva + length <= s.address + s.size);
    if (!section) throw new Error('Resource RVA does not map to section data');
    const offset = section.file + rva - section.address;
    pe.range(offset, length);
    return offset;
  }
  const base = rvaToFile(resourceRva, resourceSize);
  function resourceOffset(relative, length) {
    if (relative + length > resourceSize) throw new Error('Resource exceeds directory bounds');
    return base + relative;
  }
  function entries(relative) {
    const directory = resourceOffset(relative, 16);
    const count = pe.u16(directory + 12) + pe.u16(directory + 14);
    resourceOffset(relative + 16, count * 8);
    return Array.from({ length: count }, (_, i) => {
      const at = directory + 16 + i * 8;
      const name = pe.u32(at), target = pe.u32(at + 4);
      return { id: name & 0x80000000 ? null : name, relative: target & 0x7fffffff, directory: !!(target & 0x80000000) };
    });
  }
  const resources = [];
  function walk(relative, path = []) {
    if (path.length > 3) throw new Error('Unexpected resource depth');
    for (const entry of entries(relative)) {
      const next = [...path, entry.id];
      if (entry.directory) walk(entry.relative, next);
      else {
        const at = resourceOffset(entry.relative, 16);
        const length = pe.u32(at + 4);
        resources.push({ type: next[0], id: next[1], language: next[2], bytes: pe.range(rvaToFile(pe.u32(at), length), length) });
      }
    }
  }
  walk(0);
  const ico = view(icoBytes);
  if (ico.u16(0) !== 0 || ico.u16(2) !== 1) throw new Error('Invalid expected ICO');
  const expected = Array.from({ length: ico.u16(4) }, (_, i) => {
    const at = 6 + i * 16;
    return { descriptor: ico.range(at, 8), bytes: ico.range(ico.u32(at + 12), ico.u32(at + 8)) };
  });
  if (!expected.length) throw new Error('Expected ICO has no images');
  const groups = resources.filter(r => r.type === 14);
  if (!groups.length) throw new Error('PE has no RT_GROUP_ICON resource');
  const groupEvidence = groups.map(group => {
    const data = view(group.bytes);
    if (data.u16(0) !== 0 || data.u16(2) !== 1) throw new Error('Invalid icon group');
    const images = Array.from({ length: data.u16(4) }, (_, i) => {
      const at = 6 + i * 14;
      const id = data.u16(at + 12);
      const image = resources.find(r => r.type === 3 && r.id === id && r.language === group.language)
        ?? resources.find(r => r.type === 3 && r.id === id && r.language === 0);
      if (!image) throw new Error('Icon group references a missing RT_ICON');
      const match = expected[i];
      return { id, size: image.bytes.length, sha256: hash(image.bytes), matchesExpected: !!match && data.range(at, 8).equals(match.descriptor) && data.u32(at + 8) === image.bytes.length && image.bytes.equals(match.bytes) };
    });
    return { id: group.id, language: group.language, matchesExpected: images.length === expected.length && images.every(image => image.matchesExpected), images };
  });
  return { matchesExpected: groupEvidence.every(group => group.matchesExpected), expectedImages: expected.map(image => ({ size: image.bytes.length, sha256: hash(image.bytes) })), groups: groupEvidence };
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const exe = resolve(process.argv[2] ?? 'build/cargo/release/clickmaster.exe');
    const ico = resolve(process.argv[3] ?? 'src-tauri/icons/icon.ico');
    const evidence = inspectIcon(readFileSync(exe), readFileSync(ico));
    console.log(JSON.stringify(evidence, null, 2));
    if (!evidence.matchesExpected) { console.error('PE icon verification FAILED: group resources differ from expected ICO'); process.exitCode = 1; }
  } catch {
    console.error('PE icon verification FAILED: could not validate binary resources');
    process.exitCode = 1;
  }
}

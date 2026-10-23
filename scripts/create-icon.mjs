// Rebuild platform/frontend assets from the user-provided, repository-local PNG.
// Only essential PNG chunks are retained, excluding optional metadata.
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { resolve } from 'node:path';
import { root } from './environment.mjs';
const source = readFileSync(resolve(root, 'src-tauri/icons/source.png'));
const signature = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);
if (!source.subarray(0, 8).equals(signature)) throw new Error('Source must be a PNG');
const chunks = [signature];
for (let offset = 8; offset < source.length;) {
  const length = source.readUInt32BE(offset);
  const type = source.toString('ascii', offset + 4, offset + 8);
  const end = offset + 12 + length;
  if (end > source.length) throw new Error('Invalid PNG chunk');
  if (['IHDR', 'PLTE', 'tRNS', 'IDAT', 'IEND'].includes(type)) chunks.push(source.subarray(offset, end));
  offset = end;
}
const png = Buffer.concat(chunks);
const width = png.readUInt32BE(16);
const height = png.readUInt32BE(20);
if (width !== height || width < 16 || width > 256 || png[25] !== 6) throw new Error('Expected a square 16–256px RGBA PNG');
writeFileSync(resolve(root, 'src-tauri/icons/icon.png'), png);
mkdirSync(resolve(root, 'src/assets'), { recursive: true });
writeFileSync(resolve(root, 'src/assets/app-icon.png'), png);
const ico = Buffer.alloc(22);
ico.writeUInt16LE(1, 2); ico.writeUInt16LE(1, 4);
ico[6] = width === 256 ? 0 : width; ico[7] = height === 256 ? 0 : height;
ico.writeUInt16LE(1, 10); ico.writeUInt16LE(32, 12);
ico.writeUInt32LE(png.length, 14); ico.writeUInt32LE(22, 18);
writeFileSync(resolve(root, 'src-tauri/icons/icon.ico'), Buffer.concat([ico, png]));

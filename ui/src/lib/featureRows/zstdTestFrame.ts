const MAX_RAW_BLOCK = 128 * 1024;
const FRAME_HEADER = [0x28, 0xb5, 0x2f, 0xfd, 0x00, 0x38];

export function concatBytes(parts: Uint8Array[]): Uint8Array {
  const out = new Uint8Array(parts.reduce((sum, part) => sum + part.length, 0));
  let offset = 0;
  for (const part of parts) {
    out.set(part, offset);
    offset += part.length;
  }
  return out;
}

export function rawZstdFrame(data: Uint8Array): Uint8Array {
  const parts: Uint8Array[] = [new Uint8Array(FRAME_HEADER)];
  let offset = 0;
  do {
    const size = Math.min(MAX_RAW_BLOCK, data.length - offset);
    const last = offset + size >= data.length ? 1 : 0;
    const header = (size << 3) | last;
    parts.push(new Uint8Array([header, header >> 8, header >> 16]));
    parts.push(data.subarray(offset, offset + size));
    offset += size;
  } while (offset < data.length);
  return concatBytes(parts);
}

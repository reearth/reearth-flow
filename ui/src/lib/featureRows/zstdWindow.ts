export const MAX_ZSTD_WINDOW = 128 * 1024 * 1024;

const ZSTD_MAGIC = 0xfd2fb528;
const SKIPPABLE_MAGIC = 0x184d2a50;

type State = "magic" | "frameHeader" | "skippableSize" | "block";

const littleEndian = (bytes: number[], from: number, length: number) => {
  let value = 0;
  for (let i = length - 1; i >= 0; i--) value = value * 256 + bytes[from + i];
  return value;
};

export class ZstdWindowGuard {
  private readonly maxWindow: number;
  private state: State = "magic";
  private buffer: number[] = [];
  private need = 4;
  private skip = 0;
  private checksum = false;

  constructor(maxWindow = MAX_ZSTD_WINDOW) {
    this.maxWindow = maxWindow;
  }

  push(chunk: Uint8Array) {
    let pos = 0;
    while (pos < chunk.length) {
      if (this.skip > 0) {
        const skipped = Math.min(this.skip, chunk.length - pos);
        this.skip -= skipped;
        pos += skipped;
        continue;
      }
      this.buffer.push(chunk[pos++]);
      if (this.buffer.length >= this.need) this.consume();
    }
  }

  private expect(state: State, need: number, skip = 0) {
    this.state = state;
    this.need = need;
    this.skip = skip;
    this.buffer = [];
  }

  private consume() {
    const bytes = this.buffer;
    switch (this.state) {
      case "magic": {
        const magic = littleEndian(bytes, 0, 4);
        if (magic === ZSTD_MAGIC) {
          this.state = "frameHeader";
          this.need = 5;
          return;
        }
        if (magic >= SKIPPABLE_MAGIC && magic <= SKIPPABLE_MAGIC + 15) {
          this.expect("skippableSize", 4);
          return;
        }
        throw new Error("Not a Zstandard stream");
      }
      case "frameHeader": {
        const descriptor = bytes[4];
        const singleSegment = (descriptor >> 5) & 1;
        const dictionaryFlag = descriptor & 3;
        const sizeFlag = descriptor >> 6;
        const dictionaryBytes = dictionaryFlag === 3 ? 4 : dictionaryFlag;
        const sizeBytes = sizeFlag ? 1 << sizeFlag : singleSegment;
        const headerBytes =
          5 + (singleSegment ? 0 : 1) + dictionaryBytes + sizeBytes;
        if (bytes.length < headerBytes) {
          this.need = headerBytes;
          return;
        }

        let window: number;
        if (singleSegment) {
          window =
            littleEndian(bytes, 5 + dictionaryBytes, sizeBytes) +
            (sizeBytes === 2 ? 256 : 0);
        } else {
          const base = 2 ** (10 + (bytes[5] >> 3));
          window = base + (base / 8) * (bytes[5] & 7);
        }
        if (window > this.maxWindow) {
          throw new Error(
            `Zstandard window of ${window} bytes exceeds the ${this.maxWindow}-byte limit`,
          );
        }
        this.checksum = ((descriptor >> 2) & 1) === 1;
        this.expect("block", 3);
        return;
      }
      case "skippableSize":
        this.expect("magic", 4, littleEndian(bytes, 0, 4));
        return;
      case "block": {
        const header = littleEndian(bytes, 0, 3);
        const last = header & 1;
        const type = (header >> 1) & 3;
        if (type === 3) throw new Error("Corrupt Zstandard block");
        const payload = type === 1 ? 1 : header >>> 3;
        if (last) this.expect("magic", 4, payload + (this.checksum ? 4 : 0));
        else this.expect("block", 3, payload);
        return;
      }
    }
  }
}

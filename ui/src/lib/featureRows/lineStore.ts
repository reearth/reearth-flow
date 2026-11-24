import { scanFeature } from "./featureScan";
import type { Span } from "./featureScan";

const NEWLINE = 10;
const CARRIAGE_RETURN = 13;
const CLOSE_BRACE = 125;
const SAMPLE_SIZE = 10;
const INDEX_ARRAYS = 3;

export const DEFAULT_CHUNK_BYTES = 16 * 1024 * 1024;
const FIRST_CHUNK_BYTES = 64 * 1024;
export const DEFAULT_BYTE_BUDGET = 512 * 1024 * 1024;

export type LineStoreOptions = {
  byteBudget?: number;
  chunkBytes?: number;
};

export type ParsedRow = {
  id?: unknown;
  attributes?: Record<string, unknown>;
  geometry?: unknown;
};

export type TableRow = {
  id: unknown;
  attributes: Record<string, unknown>;
};

function isWhitespace(byte: number) {
  return byte === 32 || byte === 9 || byte === CARRIAGE_RETURN;
}

type Index = Uint32Array<ArrayBuffer>;

const EMPTY_INDEX: Index = new Uint32Array(0);
const EMPTY_BYTES = new Uint8Array(0);

function grownLength(array: Index, min: number) {
  return array.length >= min ? array.length : Math.max(min, array.length * 2);
}

function grow(array: Index, min: number): Index {
  const length = grownLength(array, min);
  if (length === array.length) return array;
  const next = new Uint32Array(length);
  next.set(array);
  return next;
}

export class LineStore {
  readonly byteBudget: number;
  private readonly chunkBytes: number;
  private readonly decoder = new TextDecoder("utf-8");

  private chunks: Uint8Array[] = [];
  private writePos = 0;
  private lineStart = 0;
  private allocatedBytes = 0;
  private sampleBytes = 0;

  private rowChunk: Index = EMPTY_INDEX;
  private rowStart: Index = EMPTY_INDEX;
  private rowLength: Index = EMPTY_INDEX;

  rowCount = 0;
  droppedRows = 0;
  skippedLines = 0;
  truncated = false;
  columns: readonly string[] = [];
  readonly sample: ParsedRow[] = [];

  private readonly columnSet = new Set<string>();
  private readonly columnList: string[] = [];
  private droppedLineHasContent = false;

  constructor(options: LineStoreOptions = {}) {
    this.byteBudget = options.byteBudget ?? DEFAULT_BYTE_BUDGET;
    this.chunkBytes = options.chunkBytes ?? DEFAULT_CHUNK_BYTES;
  }

  get totalRows() {
    return this.rowCount + this.droppedRows;
  }

  get bytes() {
    return (
      this.allocatedBytes +
      this.rowChunk.byteLength * INDEX_ARRAYS +
      this.sampleBytes
    );
  }

  append(bytes: Uint8Array) {
    const queue = [bytes];
    for (let pending = queue.shift(); pending; pending = queue.shift()) {
      if (pending.length === 0) continue;
      if (this.truncated) {
        this.countDropped(pending);
        continue;
      }

      let cut = pending.length;
      if (!this.reserve(cut)) {
        cut = this.fittingPrefix(pending);
        if (cut === 0 || !this.reserve(cut)) {
          this.truncate(pending);
          continue;
        }
      }
      const unread = this.write(pending.subarray(0, cut));
      queue.unshift(unread, pending.subarray(cut));
    }
    this.publishColumns();
  }

  finish() {
    if (
      !this.truncated &&
      this.chunks.length &&
      this.writePos > this.lineStart
    ) {
      if (
        this.addLine(
          this.chunks.length - 1,
          this.lineStart,
          this.writePos,
          this.writePos,
        )
      ) {
        this.lineStart = this.writePos;
      } else {
        this.truncate(EMPTY_BYTES);
      }
    }
    if (this.truncated) {
      if (this.droppedLineHasContent) this.droppedRows++;
      this.droppedLineHasContent = false;
    }
    this.publishColumns();
  }

  readLine(row: number): string | null {
    if (row < 0 || row >= this.rowCount) return null;
    const [chunk, start, end] = this.locate(row);
    return this.decoder.decode(chunk.subarray(start, end));
  }

  tableRow(row: number): TableRow | null {
    if (row < 0 || row >= this.rowCount) return null;
    const [chunk, start, end] = this.locate(row);
    const spans = scanFeature(chunk, start, end);
    if (!spans) return null;
    const attributes = this.parseSpan(chunk, spans.attributes);
    return {
      id: this.parseSpan(chunk, spans.id),
      attributes:
        attributes && typeof attributes === "object"
          ? (attributes as Record<string, unknown>)
          : {},
    };
  }

  private locate(row: number): [Uint8Array, number, number] {
    const start = this.rowStart[row];
    return [
      this.chunks[this.rowChunk[row]],
      start,
      start + this.rowLength[row],
    ];
  }

  private parseSpan(chunk: Uint8Array, span: Span | null): unknown {
    if (!span) return undefined;
    return JSON.parse(this.decoder.decode(chunk.subarray(span[0], span[1])));
  }

  private write(bytes: Uint8Array): Uint8Array {
    const chunkIndex = this.chunks.length - 1;
    const chunk = this.chunks[chunkIndex];
    const scanFrom = this.writePos;
    chunk.set(bytes, this.writePos);
    this.writePos += bytes.length;

    const written = chunk.subarray(0, this.writePos);
    let newline = written.indexOf(NEWLINE, scanFrom);
    while (newline !== -1) {
      const added = this.addLine(
        chunkIndex,
        this.lineStart,
        newline,
        newline + 1,
      );
      if (!added) {
        this.writePos = this.lineStart;
        this.truncate(written.subarray(this.lineStart));
        return EMPTY_BYTES;
      }
      this.lineStart = newline + 1;
      if (this.chunks[chunkIndex].length < this.writePos) {
        this.writePos = this.lineStart;
        return written.subarray(this.lineStart);
      }
      newline = written.indexOf(NEWLINE, this.lineStart);
    }
    return EMPTY_BYTES;
  }

  private truncate(rest: Uint8Array) {
    this.truncated = true;
    this.droppedLineHasContent = false;
    const current = this.chunks[this.chunks.length - 1];
    if (current) {
      this.countDropped(current.subarray(this.lineStart, this.writePos));
      this.writePos = this.lineStart;
    }
    this.countDropped(rest);
  }

  private fittingPrefix(bytes: Uint8Array) {
    let newline = bytes.lastIndexOf(NEWLINE);
    while (newline >= 0) {
      if (this.canReserve(newline + 1)) return newline + 1;
      if (newline === 0) return 0;
      newline = bytes.lastIndexOf(NEWLINE, newline - 1);
    }
    return 0;
  }

  private partialLength() {
    return this.chunks.length ? this.writePos - this.lineStart : 0;
  }

  private available(replacing: boolean) {
    const current = this.chunks[this.chunks.length - 1];
    const freed = replacing && current ? current.length : 0;
    return this.byteBudget - this.bytes + freed;
  }

  private replacingCurrent() {
    return this.chunks.length > 0 && !this.chunkHasRows(this.chunks.length - 1);
  }

  private canReserve(incoming: number) {
    const current = this.chunks[this.chunks.length - 1];
    if (current && this.writePos + incoming <= current.length) return true;
    const needed = this.partialLength() + incoming;
    return needed <= this.available(this.replacingCurrent());
  }

  private reserve(incoming: number): boolean {
    const current = this.chunks[this.chunks.length - 1];
    if (current && this.writePos + incoming <= current.length) return true;

    const partial = current
      ? current.subarray(this.lineStart, this.writePos)
      : new Uint8Array(0);
    const needed = partial.length + incoming;
    const replacing = this.replacingCurrent();
    const available = this.available(replacing);
    if (needed > available) return false;

    const grown = Math.max(
      needed,
      current
        ? current.length * 2
        : Math.min(this.chunkBytes, FIRST_CHUNK_BYTES),
    );
    const target =
      needed > this.chunkBytes ? grown : Math.min(grown, this.chunkBytes);
    const size = Math.min(target, available);

    const next = new Uint8Array(size);
    next.set(partial);
    const freed = replacing && current ? current.length : 0;
    if (replacing) this.chunks.pop();
    this.chunks.push(next);
    this.allocatedBytes += size - freed;
    this.writePos = partial.length;
    this.lineStart = 0;
    return true;
  }

  private trimCurrentChunk(keep: number) {
    const last = this.chunks.length - 1;
    const chunk = this.chunks[last];
    if (!chunk || chunk.length <= keep) return;
    this.chunks[last] = chunk.slice(0, keep);
    this.allocatedBytes -= chunk.length - keep;
  }

  private chunkHasRows(chunkIndex: number) {
    return this.rowCount > 0 && this.rowChunk[this.rowCount - 1] === chunkIndex;
  }

  private countDropped(bytes: Uint8Array) {
    let pos = 0;
    while (pos < bytes.length) {
      const newline = bytes.indexOf(NEWLINE, pos);
      const end = newline === -1 ? bytes.length : newline;
      if (!this.droppedLineHasContent) {
        for (let i = pos; i < end; i++) {
          if (!isWhitespace(bytes[i])) {
            this.droppedLineHasContent = true;
            break;
          }
        }
      }
      if (newline === -1) return;
      if (this.droppedLineHasContent) this.droppedRows++;
      this.droppedLineHasContent = false;
      pos = newline + 1;
    }
  }

  private addLine(
    chunkIndex: number,
    start: number,
    end: number,
    lineEnd: number,
  ): boolean {
    const chunk = this.chunks[chunkIndex];
    if (end > start && chunk[end - 1] === CARRIAGE_RETURN) end--;

    if (
      chunk[start] === 0xef &&
      chunk[start + 1] === 0xbb &&
      chunk[start + 2] === 0xbf &&
      start + 3 <= end
    ) {
      start += 3;
    }

    let first = start;
    while (first < end && isWhitespace(chunk[first])) first++;
    if (first === end) return true;

    const index = this.rowCount;
    const capacity = grownLength(this.rowChunk, index + 1);
    const indexGrowth = (capacity - this.rowChunk.length) * 4 * INDEX_ARRAYS;
    const sampling = this.sample.length < SAMPLE_SIZE;
    const sampleGrowth = sampling ? end - start : 0;
    if (this.bytes + indexGrowth + sampleGrowth > this.byteBudget) {
      this.trimCurrentChunk(lineEnd);
      if (this.bytes + indexGrowth + sampleGrowth > this.byteBudget) {
        return false;
      }
    }

    const attributes = sampling
      ? this.parseSample(chunk, start, end)
      : this.parseAttributes(chunk, first, end);
    if (attributes === undefined) {
      this.skippedLines++;
      return true;
    }
    if (attributes && typeof attributes === "object") {
      for (const key of Object.keys(attributes)) {
        if (this.columnSet.has(key)) continue;
        this.columnSet.add(key);
        this.columnList.push(key);
      }
    }

    this.rowChunk = grow(this.rowChunk, index + 1);
    this.rowStart = grow(this.rowStart, index + 1);
    this.rowLength = grow(this.rowLength, index + 1);
    this.rowChunk[index] = chunkIndex;
    this.rowStart[index] = start;
    this.rowLength[index] = end - start;
    this.rowCount++;
    return true;
  }

  private publishColumns() {
    if (this.columns.length !== this.columnList.length) {
      this.columns = this.columnList.slice();
    }
  }

  private parseSample(chunk: Uint8Array, start: number, end: number) {
    let row: unknown;
    try {
      row = JSON.parse(this.decoder.decode(chunk.subarray(start, end)));
    } catch {
      return undefined;
    }
    if (typeof row !== "object" || row === null || Array.isArray(row)) {
      return undefined;
    }
    this.sample.push(row as ParsedRow);
    this.sampleBytes += end - start;
    return (row as ParsedRow).attributes ?? null;
  }

  private parseAttributes(chunk: Uint8Array, start: number, end: number) {
    let last = end - 1;
    while (last > start && isWhitespace(chunk[last])) last--;
    if (chunk[last] !== CLOSE_BRACE) return undefined;
    const spans = scanFeature(chunk, start, end);
    if (!spans) return undefined;
    return this.parseSpan(chunk, spans.attributes) ?? null;
  }
}

export class RowFilter {
  readonly term: string;
  matches: Index = EMPTY_INDEX;
  count = 0;
  scanned = 0;

  private readonly matchesNormalizedValues: boolean;

  constructor(term: string) {
    this.term = term.toLowerCase();
    this.matchesNormalizedValues =
      /^[\d.eE+-]+$/.test(this.term) && /\d/.test(this.term);
  }

  get bytes() {
    return this.matches.byteLength;
  }

  caughtUp(store: LineStore) {
    return this.scanned >= store.rowCount;
  }

  advance(store: LineStore, shouldYield: () => boolean) {
    while (this.scanned < store.rowCount) {
      const row = this.scanned++;
      if (this.rowMatches(store, row)) {
        this.matches = grow(this.matches, this.count + 1);
        this.matches[this.count++] = row;
      }
      if (shouldYield()) return;
    }
  }

  rowAt(viewIndex: number): number {
    return viewIndex < this.count ? this.matches[viewIndex] : -1;
  }

  indexOf(row: number): number {
    let low = 0;
    let high = this.count - 1;
    while (low <= high) {
      const mid = (low + high) >>> 1;
      const value = this.matches[mid];
      if (value === row) return mid;
      if (value < row) low = mid + 1;
      else high = mid - 1;
    }
    return -1;
  }

  private rowMatches(store: LineStore, row: number): boolean {
    const line = store.readLine(row);
    if (line === null) return false;
    if (
      !this.matchesNormalizedValues &&
      !line.includes("\\") &&
      !line.toLowerCase().includes(this.term)
    ) {
      return false;
    }

    const parsed = store.tableRow(row);
    if (!parsed) return false;

    if (this.contains(parsed.id || row)) return true;
    for (const value of Object.values(parsed.attributes)) {
      if (this.contains(value ?? null)) return true;
    }
    return false;
  }

  private contains(value: unknown): boolean {
    let text: string;
    try {
      text = JSON.stringify(value) ?? String(value);
    } catch {
      text = String(value);
    }
    return text.toLowerCase().includes(this.term);
  }
}

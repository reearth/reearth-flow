const NEWLINE = 10;
const CARRIAGE_RETURN = 13;
const SAMPLE_SIZE = 10;

export const DEFAULT_CHUNK_BYTES = 16 * 1024 * 1024;
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

function isWhitespace(byte: number) {
  return byte === 32 || byte === 9 || byte === CARRIAGE_RETURN;
}

type Index = Uint32Array<ArrayBuffer>;

function grow(array: Index, min: number): Index {
  if (array.length >= min) return array;
  const next = new Uint32Array(Math.max(min, array.length * 2));
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

  private rowChunk: Index = new Uint32Array(1024);
  private rowStart: Index = new Uint32Array(1024);
  private rowLength: Index = new Uint32Array(1024);

  rowCount = 0;
  droppedRows = 0;
  skippedLines = 0;
  truncated = false;
  columns: string[] = [];
  readonly sample: ParsedRow[] = [];

  private readonly columnSet = new Set<string>();
  private droppedLineHasContent = false;

  constructor(options: LineStoreOptions = {}) {
    this.byteBudget = options.byteBudget ?? DEFAULT_BYTE_BUDGET;
    this.chunkBytes = options.chunkBytes ?? DEFAULT_CHUNK_BYTES;
  }

  get totalRows() {
    return this.rowCount + this.droppedRows;
  }

  get bytes() {
    return this.allocatedBytes;
  }

  append(bytes: Uint8Array) {
    if (bytes.length === 0) return;
    if (this.truncated || !this.reserve(bytes.length)) {
      this.countDropped(bytes);
      return;
    }

    const chunkIndex = this.chunks.length - 1;
    const chunk = this.chunks[chunkIndex];
    const scanFrom = this.writePos;
    chunk.set(bytes, this.writePos);
    this.writePos += bytes.length;

    const written = chunk.subarray(0, this.writePos);
    let newline = written.indexOf(NEWLINE, scanFrom);
    while (newline !== -1) {
      this.addLine(chunkIndex, this.lineStart, newline);
      this.lineStart = newline + 1;
      newline = written.indexOf(NEWLINE, this.lineStart);
    }
  }

  finish() {
    if (this.truncated) {
      if (this.droppedLineHasContent) this.droppedRows++;
      this.droppedLineHasContent = false;
      return;
    }
    if (this.chunks.length && this.writePos > this.lineStart) {
      this.addLine(this.chunks.length - 1, this.lineStart, this.writePos);
      this.lineStart = this.writePos;
    }
  }

  readLine(row: number): string | null {
    if (row < 0 || row >= this.rowCount) return null;
    const chunk = this.chunks[this.rowChunk[row]];
    const start = this.rowStart[row];
    return this.decoder.decode(
      chunk.subarray(start, start + this.rowLength[row]),
    );
  }

  parseRow(row: number): ParsedRow | null {
    const line = this.readLine(row);
    if (line === null) return null;
    try {
      return JSON.parse(line);
    } catch {
      return null;
    }
  }

  private reserve(incoming: number): boolean {
    const current = this.chunks[this.chunks.length - 1];
    if (current && this.writePos + incoming <= current.length) return true;

    const partial = current
      ? current.subarray(this.lineStart, this.writePos)
      : new Uint8Array(0);
    const needed = partial.length + incoming;
    const size =
      needed > this.chunkBytes
        ? Math.max(needed, (current?.length ?? 0) * 2)
        : this.chunkBytes;
    const replacing = !!current && !this.chunkHasRows(this.chunks.length - 1);
    const freed = replacing ? current.length : 0;

    if (this.allocatedBytes - freed + size > this.byteBudget) {
      this.truncated = true;
      this.droppedLineHasContent = partial.some((byte) => !isWhitespace(byte));
      return false;
    }

    const next = new Uint8Array(size);
    next.set(partial);
    if (replacing) this.chunks.pop();
    this.chunks.push(next);
    this.allocatedBytes += size - freed;
    this.writePos = partial.length;
    this.lineStart = 0;
    return true;
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

  private addLine(chunkIndex: number, start: number, end: number) {
    const chunk = this.chunks[chunkIndex];
    if (end > start && chunk[end - 1] === CARRIAGE_RETURN) end--;

    let first = start;
    while (first < end && isWhitespace(chunk[first])) first++;
    if (first === end) return;

    let parsed: unknown;
    try {
      parsed = JSON.parse(this.decoder.decode(chunk.subarray(start, end)));
    } catch {
      this.skippedLines++;
      return;
    }
    if (typeof parsed !== "object" || parsed === null) {
      this.skippedLines++;
      return;
    }

    const row = parsed as ParsedRow;
    const attributes = row.attributes;
    if (attributes && typeof attributes === "object") {
      for (const key of Object.keys(attributes)) {
        if (this.columnSet.has(key)) continue;
        this.columnSet.add(key);
        this.columns = [...this.columns, key];
      }
    }
    if (this.sample.length < SAMPLE_SIZE) this.sample.push(row);

    const index = this.rowCount;
    this.rowChunk = grow(this.rowChunk, index + 1);
    this.rowStart = grow(this.rowStart, index + 1);
    this.rowLength = grow(this.rowLength, index + 1);
    this.rowChunk[index] = chunkIndex;
    this.rowStart[index] = start;
    this.rowLength[index] = end - start;
    this.rowCount++;
  }
}

export class RowFilter {
  readonly term: string;
  matches: Index = new Uint32Array(1024);
  count = 0;
  scanned = 0;

  private readonly matchesNormalizedValues: boolean;

  constructor(term: string) {
    this.term = term.toLowerCase();
    this.matchesNormalizedValues = /^[\d.eE+-]+$/.test(this.term);
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

    let parsed: ParsedRow;
    try {
      parsed = JSON.parse(line);
    } catch {
      return false;
    }

    if (this.contains(parsed.id || row)) return true;
    for (const value of Object.values(parsed.attributes ?? {})) {
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

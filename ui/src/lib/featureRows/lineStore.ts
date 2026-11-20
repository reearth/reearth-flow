const NEWLINE = 10;
const CARRIAGE_RETURN = 13;
const QUOTE = 34;
const BACKSLASH = 92;
const COMMA = 44;
const COLON = 58;
const OPEN_BRACE = 123;
const CLOSE_BRACE = 125;
const OPEN_BRACKET = 91;
const CLOSE_BRACKET = 93;
const ATTRIBUTES_KEY = new TextEncoder().encode('"attributes"');
const SAMPLE_SIZE = 10;

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

function isWhitespace(byte: number) {
  return byte === 32 || byte === 9 || byte === CARRIAGE_RETURN;
}

function skipWhitespace(bytes: Uint8Array, pos: number, end: number) {
  while (pos < end && isWhitespace(bytes[pos])) pos++;
  return pos;
}

function skipString(bytes: Uint8Array, pos: number, end: number) {
  for (let i = pos + 1; i < end; i++) {
    if (bytes[i] === BACKSLASH) i++;
    else if (bytes[i] === QUOTE) return i + 1;
  }
  return -1;
}

function valueEnd(bytes: Uint8Array, pos: number, end: number) {
  let depth = 0;
  let i = pos;
  while (i < end) {
    const byte = bytes[i];
    if (byte === QUOTE) {
      i = skipString(bytes, i, end);
      if (i < 0 || depth === 0) return i;
      continue;
    }
    if (byte === OPEN_BRACE || byte === OPEN_BRACKET) {
      depth++;
    } else if (byte === CLOSE_BRACE || byte === CLOSE_BRACKET) {
      if (depth === 0) return i;
      if (--depth === 0) return i + 1;
    } else if (depth === 0 && byte === COMMA) {
      return i;
    }
    i++;
  }
  return depth === 0 ? end : -1;
}

function isAttributesKey(bytes: Uint8Array, start: number, end: number) {
  if (end - start !== ATTRIBUTES_KEY.length) return false;
  for (let i = 0; i < ATTRIBUTES_KEY.length; i++) {
    if (bytes[start + i] !== ATTRIBUTES_KEY[i]) return false;
  }
  return true;
}

function attributesSpan(
  bytes: Uint8Array,
  start: number,
  end: number,
): [number, number] | null {
  let depth = 0;
  let i = start;
  while (i < end) {
    const byte = bytes[i];
    if (byte === QUOTE) {
      const keyStart = i;
      i = skipString(bytes, i, end);
      if (i < 0) return null;
      if (depth === 1 && isAttributesKey(bytes, keyStart, i)) {
        const colon = skipWhitespace(bytes, i, end);
        if (bytes[colon] === COLON) {
          const from = skipWhitespace(bytes, colon + 1, end);
          const to = valueEnd(bytes, from, end);
          return to < 0 ? null : [from, to];
        }
      }
      continue;
    }
    if (byte === OPEN_BRACE || byte === OPEN_BRACKET) depth++;
    else if (byte === CLOSE_BRACE || byte === CLOSE_BRACKET) depth--;
    i++;
  }
  return null;
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
    const replacing = !!current && !this.chunkHasRows(this.chunks.length - 1);
    const freed = replacing ? current.length : 0;
    const available = this.byteBudget - this.allocatedBytes + freed;

    if (needed > available) {
      this.truncated = true;
      this.droppedLineHasContent = partial.some((byte) => !isWhitespace(byte));
      return false;
    }

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

    let last = end - 1;
    while (last > first && isWhitespace(chunk[last])) last--;
    if (chunk[first] !== OPEN_BRACE || chunk[last] !== CLOSE_BRACE) {
      this.skippedLines++;
      return;
    }

    const attributes =
      this.sample.length < SAMPLE_SIZE
        ? this.parseSample(chunk, start, end)
        : this.parseAttributes(chunk, first, end);
    if (attributes === undefined) {
      this.skippedLines++;
      return;
    }
    if (attributes && typeof attributes === "object") {
      for (const key of Object.keys(attributes)) {
        if (this.columnSet.has(key)) continue;
        this.columnSet.add(key);
        this.columns = [...this.columns, key];
      }
    }

    const index = this.rowCount;
    this.rowChunk = grow(this.rowChunk, index + 1);
    this.rowStart = grow(this.rowStart, index + 1);
    this.rowLength = grow(this.rowLength, index + 1);
    this.rowChunk[index] = chunkIndex;
    this.rowStart[index] = start;
    this.rowLength[index] = end - start;
    this.rowCount++;
  }

  private parseSample(chunk: Uint8Array, start: number, end: number) {
    let row: ParsedRow;
    try {
      row = JSON.parse(this.decoder.decode(chunk.subarray(start, end)));
    } catch {
      return undefined;
    }
    this.sample.push(row);
    return row.attributes ?? null;
  }

  private parseAttributes(chunk: Uint8Array, start: number, end: number) {
    const span = attributesSpan(chunk, start, end);
    if (!span) return null;
    try {
      return JSON.parse(
        this.decoder.decode(chunk.subarray(span[0], span[1])),
      ) as unknown;
    } catch {
      return undefined;
    }
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
    this.matchesNormalizedValues =
      /^[\d.eE+-]+$/.test(this.term) && /\d/.test(this.term);
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

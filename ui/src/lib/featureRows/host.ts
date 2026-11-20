import { Decompress } from "fzstd";

import { analyzeDataType } from "./geometryType";
import { LineStore, RowFilter } from "./lineStore";
import type { LineStoreOptions } from "./lineStore";
import type {
  FeatureRowRecord,
  StatusMessage,
  WorkerRequest,
  WorkerResponse,
} from "./protocol";
import { ZstdWindowGuard } from "./zstdWindow";

const STATUS_INTERVAL_MS = 150;
const SEARCH_SLICE_MS = 12;
const MAX_ROWS_PER_REQUEST = 500;
const MAX_CACHED_FILES = 3;
const CACHED_BYTE_BUDGET = 1024 * 1024 * 1024;
const INACTIVE_BYTE_BUDGET = 256 * 1024 * 1024;

type Entry = {
  url: string;
  store: LineStore;
  controller: AbortController;
  done: boolean;
  error: string | null;
  filter: RowFilter | null;
  searchTerm: string;
  geometryType: string | null;
  geometryResolved: boolean;
  lastUsed: number;
  postedColumns: readonly string[] | null;
  lastPostAt: number;
  statusTimer: ReturnType<typeof setTimeout> | null;
  pumping: boolean;
};

export type HostOptions = {
  post: (message: WorkerResponse) => void;
  fetch?: typeof fetch;
  store?: LineStoreOptions;
  now?: () => number;
};

const yieldToEvents = () => new Promise((resolve) => setTimeout(resolve, 0));

const isCompressed = (url: string) =>
  url.split("?")[0].toLowerCase().endsWith(".zst");

export class FeatureRowsHost {
  private readonly post: HostOptions["post"];
  private readonly fetchFn: typeof fetch;
  private readonly storeOptions?: LineStoreOptions;
  private readonly now: () => number;
  private readonly entries = new Map<string, Entry>();
  private active: string | null = null;
  private clock = 0;

  constructor(options: HostOptions) {
    this.post = options.post;
    this.fetchFn = options.fetch ?? ((...args) => fetch(...args));
    this.storeOptions = options.store;
    this.now = options.now ?? (() => performance.now());
  }

  handle(request: WorkerRequest) {
    switch (request.type) {
      case "open":
        this.open(request.url);
        return;
      case "search":
        this.search(request.url, request.term);
        return;
      case "rows":
        this.post({
          type: "rows",
          id: request.id,
          searchTerm: this.entries.get(request.url)?.searchTerm ?? "",
          rows: this.rows(request.url, request.start, request.count),
        });
        return;
      case "line":
        this.post({
          type: "line",
          id: request.id,
          text:
            this.entries.get(request.url)?.store.readLine(request.row) ?? null,
        });
        return;
      case "viewIndex":
        this.post({
          type: "viewIndex",
          id: request.id,
          index: this.viewIndex(request.url, request.row),
        });
        return;
    }
  }

  async settled(url: string) {
    while (true) {
      const entry = this.entries.get(url);
      if (!entry || (entry.done && !entry.pumping)) return;
      await yieldToEvents();
    }
  }

  private open(url: string) {
    this.active = url;

    for (const other of [...this.entries.values()]) {
      if (other.url !== url && !other.done) this.discard(other);
    }

    let entry = this.entries.get(url);
    if (entry?.error) {
      this.discard(entry);
      entry = undefined;
    }
    if (!entry) {
      entry = this.create(url);
      this.entries.set(url, entry);
      void this.load(entry);
    }

    entry.lastUsed = ++this.clock;
    entry.postedColumns = null;
    this.evict();
    this.postStatus(entry, true);
  }

  private search(url: string, rawTerm: string) {
    const entry = this.entries.get(url);
    if (!entry) return;
    const term = rawTerm.trim();
    if (term === entry.searchTerm) return;

    entry.searchTerm = term;
    entry.filter = term ? new RowFilter(term) : null;
    this.postStatus(entry, true);
    if (entry.filter) void this.pump(entry);
  }

  private rows(url: string, start: number, count: number) {
    const entry = this.entries.get(url);
    if (!entry) return [];
    const { store, filter } = entry;
    const viewCount = filter ? filter.count : store.rowCount;
    const end = Math.min(
      viewCount,
      start + Math.min(count, MAX_ROWS_PER_REQUEST),
    );

    const rows: FeatureRowRecord[] = [];
    for (let index = Math.max(0, start); index < end; index++) {
      const row = filter ? filter.rowAt(index) : index;
      const parsed = store.tableRow(row);
      rows.push({
        row,
        id: parsed?.id,
        attributes: parsed?.attributes ?? {},
      });
    }
    return rows;
  }

  private viewIndex(url: string, row: number) {
    const entry = this.entries.get(url);
    if (!entry) return -1;
    if (entry.filter) return entry.filter.indexOf(row);
    return row >= 0 && row < entry.store.rowCount ? row : -1;
  }

  private create(url: string): Entry {
    return {
      url,
      store: new LineStore(this.storeOptions),
      controller: new AbortController(),
      done: false,
      error: null,
      filter: null,
      searchTerm: "",
      geometryType: null,
      geometryResolved: false,
      lastUsed: 0,
      postedColumns: null,
      lastPostAt: -Infinity,
      statusTimer: null,
      pumping: false,
    };
  }

  private discard(entry: Entry) {
    entry.controller.abort();
    if (entry.statusTimer) clearTimeout(entry.statusTimer);
    this.entries.delete(entry.url);
  }

  private evict() {
    const candidates = [...this.entries.values()]
      .filter((entry) => entry.url !== this.active && entry.done)
      .sort((a, b) => a.lastUsed - b.lastUsed);

    const sizeOf = (entry: Entry) =>
      entry.store.bytes + (entry.filter?.bytes ?? 0);
    let bytes = 0;
    for (const entry of this.entries.values()) bytes += sizeOf(entry);
    let inactiveBytes = 0;
    for (const entry of candidates) inactiveBytes += sizeOf(entry);

    for (const entry of candidates) {
      if (
        this.entries.size <= MAX_CACHED_FILES &&
        bytes <= CACHED_BYTE_BUDGET &&
        inactiveBytes <= INACTIVE_BYTE_BUDGET
      ) {
        return;
      }
      bytes -= sizeOf(entry);
      inactiveBytes -= sizeOf(entry);
      this.discard(entry);
    }
  }

  private async load(entry: Entry) {
    const { store, controller } = entry;
    try {
      const response = await this.fetchFn(entry.url, {
        signal: controller.signal,
      });
      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`);
      }
      const reader = response.body?.getReader();
      if (!reader) throw new Error("Response body is not readable");

      const compressed = isCompressed(entry.url);
      const guard = compressed ? new ZstdWindowGuard() : null;
      const decompressor = compressed
        ? new Decompress((chunk) => store.append(chunk))
        : null;

      try {
        while (true) {
          const { done, value } = await reader.read();
          if (controller.signal.aborted) return;
          if (done) break;
          if (decompressor) {
            guard?.push(value);
            decompressor.push(value);
          } else {
            store.append(value);
          }
          this.afterAppend(entry);
        }
      } catch (error) {
        await reader.cancel().catch(() => undefined);
        throw error;
      } finally {
        reader.releaseLock();
      }

      decompressor?.push(new Uint8Array(0), true);
      store.finish();
    } catch (error) {
      if (controller.signal.aborted) return;
      entry.error = error instanceof Error ? error.message : String(error);
    }

    entry.done = true;
    this.resolveGeometryType(entry);
    this.postStatus(entry, true);
    if (entry.filter) void this.pump(entry);
    this.evict();
  }

  private afterAppend(entry: Entry) {
    this.resolveGeometryType(entry);
    this.postStatus(entry);
    if (entry.filter) void this.pump(entry);
  }

  private resolveGeometryType(entry: Entry) {
    if (entry.geometryResolved) return;
    const { sample } = entry.store;
    if (sample.length < 10 && !entry.done) return;
    entry.geometryType = analyzeDataType(sample).geometryType;
    entry.geometryResolved = true;
  }

  private async pump(entry: Entry) {
    if (entry.pumping) return;
    entry.pumping = true;
    try {
      while (
        this.entries.get(entry.url) === entry &&
        entry.filter &&
        !entry.filter.caughtUp(entry.store)
      ) {
        const deadline = this.now() + SEARCH_SLICE_MS;
        entry.filter.advance(entry.store, () => this.now() > deadline);
        this.postStatus(entry);
        await yieldToEvents();
      }
    } finally {
      entry.pumping = false;
    }
    this.postStatus(entry, true);
  }

  private postStatus(entry: Entry, force = false) {
    if (entry.url !== this.active || this.entries.get(entry.url) !== entry) {
      return;
    }

    const now = this.now();
    if (!force && now - entry.lastPostAt < STATUS_INTERVAL_MS) {
      entry.statusTimer ??= setTimeout(() => {
        entry.statusTimer = null;
        this.postStatus(entry, true);
      }, STATUS_INTERVAL_MS);
      return;
    }

    if (entry.statusTimer) {
      clearTimeout(entry.statusTimer);
      entry.statusTimer = null;
    }
    entry.lastPostAt = now;

    const { store, filter } = entry;
    const status: StatusMessage = {
      url: entry.url,
      rowCount: store.rowCount,
      totalRows: store.totalRows,
      viewCount: filter ? filter.count : store.rowCount,
      searchTerm: entry.searchTerm,
      searching: !!filter && !filter.caughtUp(store),
      done: entry.done,
      truncated: store.truncated,
      skippedLines: store.skippedLines,
      geometryType: entry.geometryType,
      error: entry.error,
    };
    if (entry.postedColumns !== store.columns) {
      status.columns = store.columns;
      entry.postedColumns = store.columns;
    }
    this.post({ type: "status", status });
  }
}

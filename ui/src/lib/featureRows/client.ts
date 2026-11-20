import type {
  FeatureRowRecord,
  FeatureRowsStatus,
  WorkerRequest,
  WorkerResponse,
} from "./protocol";

export type WorkerLike = {
  postMessage: (message: WorkerRequest) => void;
  onmessage: ((event: MessageEvent<WorkerResponse>) => void) | null;
  onerror: ((event: ErrorEvent) => void) | null;
  onmessageerror: ((event: MessageEvent) => void) | null;
  terminate: () => void;
};

type Listener = (status: FeatureRowsStatus) => void;

type Reply = Extract<WorkerResponse, { id: number }>;

type Pending = {
  resolve: (reply: Reply) => void;
  reject: (error: Error) => void;
};

export type RowsReply = {
  searchTerm: string;
  rows: FeatureRowRecord[];
};

export class FeatureRowsClient {
  private worker: WorkerLike | null = null;
  private readonly createWorker: () => WorkerLike;
  private nextId = 1;
  private readonly pending = new Map<number, Pending>();
  private readonly listeners = new Map<string, Set<Listener>>();
  private readonly statuses = new Map<string, FeatureRowsStatus>();
  private session = 0;

  constructor(createWorker: () => WorkerLike) {
    this.createWorker = createWorker;
  }

  status(url: string): FeatureRowsStatus | undefined {
    return this.statuses.get(url);
  }

  subscribe(url: string, listener: Listener): () => void {
    let set = this.listeners.get(url);
    if (!set) {
      set = new Set();
      this.listeners.set(url, set);
    }
    set.add(listener);
    return () => {
      set.delete(listener);
      if (!set.size) this.listeners.delete(url);
    };
  }

  open(url: string) {
    this.send({ type: "open", url });
  }

  clear() {
    const terms = new Map(
      [...this.statuses].map(([url, status]) => [url, status.searchTerm]),
    );
    this.reset("The data preview was cleared");
    this.statuses.clear();
    queueMicrotask(() => {
      for (const url of this.listeners.keys()) {
        this.reopen(url, terms.get(url) ?? "");
      }
    });
  }

  retry(url: string) {
    const term = this.statuses.get(url)?.searchTerm ?? "";
    this.session++;
    this.statuses.delete(url);
    this.reopen(url, term);
  }

  private reopen(url: string, term: string) {
    this.open(url);
    if (term) this.search(url, term);
  }

  search(url: string, term: string) {
    this.send({ type: "search", url, term });
  }

  async rows(url: string, start: number, count: number): Promise<RowsReply> {
    const reply = await this.request((id) => ({
      type: "rows",
      id,
      url,
      start,
      count,
    }));
    return reply.type === "rows"
      ? { searchTerm: reply.searchTerm, rows: reply.rows }
      : { searchTerm: "", rows: [] };
  }

  async line(url: string, row: number): Promise<string | null> {
    const reply = await this.request((id) => ({ type: "line", id, url, row }));
    return reply.type === "line" ? reply.text : null;
  }

  async viewIndexOf(url: string, row: number): Promise<number> {
    const reply = await this.request((id) => ({
      type: "viewIndex",
      id,
      url,
      row,
    }));
    return reply.type === "viewIndex" ? reply.index : -1;
  }

  private ensureWorker(): WorkerLike {
    if (!this.worker) {
      const worker = this.createWorker();
      const current = () => this.worker === worker;
      worker.onmessage = (event) => {
        if (current()) this.receive(event.data);
      };
      worker.onerror = (event) => {
        if (current()) {
          this.fail(event.message || "The data preview worker failed");
        }
      };
      worker.onmessageerror = () => {
        if (current()) {
          this.fail("The data preview worker sent an unreadable message");
        }
      };
      this.worker = worker;
    }
    return this.worker;
  }

  private send(message: WorkerRequest) {
    this.ensureWorker().postMessage(message);
  }

  private request(build: (id: number) => WorkerRequest): Promise<Reply> {
    const id = this.nextId++;
    return new Promise<Reply>((resolve, reject) => {
      this.pending.set(id, { resolve, reject });
      try {
        this.send(build(id));
      } catch (error) {
        this.pending.delete(id);
        reject(error);
      }
    });
  }

  private reset(message: string) {
    this.worker?.terminate();
    this.worker = null;
    this.session++;

    const error = new Error(message);
    const pending = [...this.pending.values()];
    this.pending.clear();
    pending.forEach(({ reject }) => reject(error));
  }

  private fail(message: string) {
    this.reset(message);
    const urls = new Set([...this.statuses.keys(), ...this.listeners.keys()]);
    for (const url of urls) {
      this.setStatus({
        ...(this.statuses.get(url) ?? emptyStatus(url, this.session)),
        rowCount: 0,
        totalRows: 0,
        viewCount: 0,
        done: true,
        searching: false,
        error: message,
        session: this.session,
      });
    }
  }

  private setStatus(status: FeatureRowsStatus) {
    this.statuses.set(status.url, status);
    this.listeners.get(status.url)?.forEach((listener) => listener(status));
  }

  private receive(message: WorkerResponse) {
    if (message.type === "status") {
      const { columns, ...rest } = message.status;
      const previous = this.statuses.get(rest.url);
      this.setStatus({
        ...rest,
        columns: columns ?? previous?.columns ?? [],
        session: this.session,
      });
      return;
    }

    const pending = this.pending.get(message.id);
    if (!pending) return;
    this.pending.delete(message.id);
    pending.resolve(message);
  }
}

function emptyStatus(url: string, session: number): FeatureRowsStatus {
  return {
    url,
    rowCount: 0,
    totalRows: 0,
    viewCount: 0,
    searchTerm: "",
    searching: false,
    done: false,
    truncated: false,
    skippedLines: 0,
    columns: [],
    geometryType: null,
    error: null,
    session,
  };
}

let shared: FeatureRowsClient | null = null;

export function featureRowsClient(): FeatureRowsClient {
  shared ??= new FeatureRowsClient(
    () =>
      new Worker(new URL("./worker.ts", import.meta.url), { type: "module" }),
  );
  return shared;
}

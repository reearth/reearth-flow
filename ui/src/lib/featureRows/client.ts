import type {
  FeatureRowRecord,
  FeatureRowsStatus,
  WorkerRequest,
  WorkerResponse,
} from "./protocol";

export type WorkerLike = {
  postMessage: (message: WorkerRequest) => void;
  onmessage: ((event: MessageEvent<WorkerResponse>) => void) | null;
};

type Listener = (status: FeatureRowsStatus) => void;

type Pending = {
  resolve: (value: any) => void;
};

type DistributiveOmit<T, K extends PropertyKey> = T extends unknown
  ? Omit<T, K>
  : never;

export class FeatureRowsClient {
  private worker: WorkerLike | null = null;
  private readonly createWorker: () => WorkerLike;
  private nextId = 1;
  private readonly pending = new Map<number, Pending>();
  private readonly listeners = new Map<string, Set<Listener>>();
  private readonly statuses = new Map<string, FeatureRowsStatus>();

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

  search(url: string, term: string) {
    this.send({ type: "search", url, term });
  }

  rows(url: string, start: number, count: number): Promise<FeatureRowRecord[]> {
    return this.request({ type: "rows", url, start, count });
  }

  line(url: string, row: number): Promise<string | null> {
    return this.request({ type: "line", url, row });
  }

  viewIndexOf(url: string, row: number): Promise<number> {
    return this.request({ type: "viewIndex", url, row });
  }

  private ensureWorker(): WorkerLike {
    if (!this.worker) {
      this.worker = this.createWorker();
      this.worker.onmessage = (event) => this.receive(event.data);
    }
    return this.worker;
  }

  private send(message: WorkerRequest) {
    this.ensureWorker().postMessage(message);
  }

  private request<T>(
    message: DistributiveOmit<Extract<WorkerRequest, { id: number }>, "id">,
  ): Promise<T> {
    const id = this.nextId++;
    return new Promise<T>((resolve) => {
      this.pending.set(id, { resolve });
      this.send({ ...message, id } as WorkerRequest);
    });
  }

  private receive(message: WorkerResponse) {
    if (message.type === "status") {
      const { columns, ...rest } = message.status;
      const previous = this.statuses.get(rest.url);
      const status: FeatureRowsStatus = {
        ...rest,
        columns: columns ?? previous?.columns ?? [],
      };
      this.statuses.set(rest.url, status);
      this.listeners.get(rest.url)?.forEach((listener) => listener(status));
      return;
    }

    const pending = this.pending.get(message.id);
    if (!pending) return;
    this.pending.delete(message.id);
    if (message.type === "rows") pending.resolve(message.rows);
    else if (message.type === "line") pending.resolve(message.text);
    else pending.resolve(message.index);
  }
}

let shared: FeatureRowsClient | null = null;

export function featureRowsClient(): FeatureRowsClient {
  shared ??= new FeatureRowsClient(
    () =>
      new Worker(new URL("./worker.ts", import.meta.url), {
        type: "module",
      }) as unknown as WorkerLike,
  );
  return shared;
}

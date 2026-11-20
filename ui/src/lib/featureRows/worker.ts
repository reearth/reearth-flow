import { FeatureRowsHost } from "./host";
import type { WorkerRequest, WorkerResponse } from "./protocol";

const host = new FeatureRowsHost({
  post: (message: WorkerResponse) => self.postMessage(message),
});

self.addEventListener("message", (event: MessageEvent<WorkerRequest>) =>
  host.handle(event.data),
);

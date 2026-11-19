import { FeatureRowsHost } from "./host";
import type { WorkerRequest, WorkerResponse } from "./protocol";

const scope = self as unknown as {
  postMessage: (message: WorkerResponse) => void;
  onmessage: ((event: MessageEvent<WorkerRequest>) => void) | null;
};

const host = new FeatureRowsHost({
  post: (message) => scope.postMessage(message),
});

scope.onmessage = (event) => host.handle(event.data);
